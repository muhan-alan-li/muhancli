Register-ArgumentCompleter -Native -CommandName muhan -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    # Parse only the text before the cursor, keeping quoted strings intact.
    # Do not evaluate the user's input or pass words through native quoting.
    $length = [Math]::Max(0, [Math]::Min(
        $cursorPosition - $commandAst.Extent.StartOffset,
        $commandAst.Extent.Text.Length))
    $prefix = $commandAst.Extent.Text.Substring(0, $length)
    $tokens = $null
    $parseErrors = $null
    $tree = [System.Management.Automation.Language.Parser]::ParseInput(
        $prefix, [ref]$tokens, [ref]$parseErrors)
    $parsedCommand = $tree.Find({
        param($node)
        $node -is [System.Management.Automation.Language.CommandAst]
    }, $false)
    if ($null -eq $parsedCommand) { return }
    $words = @('muhan')
    foreach ($element in @($parsedCommand.CommandElements | Select-Object -Skip 1)) {
        if ($element -is [System.Management.Automation.Language.StringConstantExpressionAst]) {
            $words += $element.Value
        } elseif ($element -is [System.Management.Automation.Language.CommandParameterAst]) {
            $words += $element.Extent.Text
        } else {
            # Expressions such as variable expansion are not executed for Tab.
            return
        }
    }
    if ([string]::IsNullOrEmpty($wordToComplete)) { $words += '' }

    $previousShell = $env:MUHAN_COMPLETE
    $previousWords = $env:MUHAN_COMPLETE_WORDS
    try {
        $env:MUHAN_COMPLETE = 'powershell'
        $env:MUHAN_COMPLETE_WORDS = ConvertTo-Json -InputObject $words -Compress
        $results = & muhan
        foreach ($result in $results) {
            $parts = $result -split "`t", 2
            $value = $parts[0]
            # Quote candidates containing spaces or shell metacharacters.
            $text = if ($value -match '[\s`"''$;&|()<>{}]') {
                "'" + $value.Replace("'", "''") + "'"
            } else { $value }
            $help = if ($parts.Length -gt 1) { $parts[1] } else { $value }
            [System.Management.Automation.CompletionResult]::new(
                $text, $value, 'ParameterValue', $help)
        }
    } finally {
        if ($null -eq $previousShell) { Remove-Item Env:\MUHAN_COMPLETE -ErrorAction SilentlyContinue }
        else { $env:MUHAN_COMPLETE = $previousShell }
        if ($null -eq $previousWords) { Remove-Item Env:\MUHAN_COMPLETE_WORDS -ErrorAction SilentlyContinue }
        else { $env:MUHAN_COMPLETE_WORDS = $previousWords }
    }
}
