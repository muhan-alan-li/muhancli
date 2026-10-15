"""Exercise real shell completion with an isolated HOME and an installed binary."""

import argparse
import os
from pathlib import Path
import pty
import select
import shutil
import signal
import subprocess
import tempfile
import time

CASES = [
    ('muhan ', ['about', 'experience', 'projects', '--json']),
    ('muhan exp', ['experience']),
    ('muhan experience ', ['apera-ai', 'nrcan', 'ubc-cic']),
    ('muhan experience ap', ['apera-ai']),
    ('muhan projects dune', ['dune-imperium']),
    ('muhan projects --tag Re', ['React']),
    ('muhan skills --group Lang', ['Languages']),
    ('muhan --plain experience nr', ['nrcan']),
    ('muhan projects --j', ['--json']),
]


def read_until(fd, predicate, timeout=20):
    output = b''
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if select.select([fd], [], [], 0.2)[0]:
            try:
                chunk = os.read(fd, 65536)
            except OSError:
                break
            if not chunk:
                break
            output += chunk
            if predicate(output.decode(errors='replace')):
                return output.decode(errors='replace')
    raise AssertionError(f'Shell completion timed out: {output.decode(errors="replace")!r}')


def check_interactive(shell, env, directory, line, expected):
    # A controlling terminal and actual Tab keystrokes exercise readline/ZLE,
    # including handler registration and fallback to filename completion.
    pid, fd = pty.fork()
    if pid == 0:
        os.chdir(directory)
        args = ['--noprofile', '--norc', '-i'] if shell == 'bash' else ['-f', '-i']
        os.execvpe(shell, [shell, *args], env)
    try:
        if shell == 'bash':
            setup = "PS1=''; PS2=''; bind 'set show-all-if-ambiguous on'; "
        else:
            # Hosted runners can include writable directories in their system
            # fpath. Ignore those directories instead of waiting for a prompt.
            setup = "PROMPT=''; RPROMPT=''; autoload -Uz compinit; compinit -i -D; "
        setup += f'eval "$(muhan completions {shell})"; printf "\\n__MUHAN_READY__\\n"\n'
        os.write(fd, setup.encode())
        read_until(fd, lambda output: '\r\n__MUHAN_READY__\r\n' in output)
        # One Tab lists candidates with show-all-if-ambiguous in Bash and
        # default Zsh settings; a second also covers ordinary Bash behavior.
        os.write(fd, (line + '\t\t').encode())
        output = read_until(fd, lambda output: all(item in output for item in expected))
        assert 'ap-local-decoy' not in output, (shell, line, output)
        assert 'unrelated-local-decoy' not in output, (shell, line, output)
    finally:
        os.kill(pid, signal.SIGKILL)
        os.waitpid(pid, 0)
        os.close(fd)


def check_fish(env, directory, line, expected):
    # Fish exposes the same completion engine used by Tab as complete -C.
    result = subprocess.run(
        ['fish', '--no-config', '-c',
         'muhan completions fish | source; complete -C "$argv[1]"', '--', line],
        env=env, cwd=directory, text=True, capture_output=True, timeout=20,
    )
    assert result.returncode == 0, result.stderr
    candidates = [entry.split('\t')[0] for entry in result.stdout.splitlines()]
    assert all(item in candidates for item in expected), (line, candidates, expected)
    assert not any('local-decoy' in item for item in candidates), (line, candidates)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=Path('target/release/muhan'))
    parser.add_argument('--shell', choices=['bash', 'zsh', 'fish'], action='append', required=True)
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix='muhan-completion-') as root:
        directory = Path(root)
        home = directory / 'home'
        home.mkdir()
        bin_dir = directory / 'bin with spaces'
        bin_dir.mkdir()
        shutil.copy2(binary, bin_dir / 'muhan')
        (directory / 'ap-local-decoy').touch()
        (directory / 'unrelated-local-decoy').touch()
        env = dict(os.environ, HOME=str(home), ZDOTDIR=str(home), TERM='dumb',
                   XDG_CONFIG_HOME=str(home / '.config'),
                   PATH=str(bin_dir) + os.pathsep + os.environ['PATH'])
        for key in ['MUHAN_COMPLETE', 'BASH_ENV', 'ENV']:
            env.pop(key, None)
        for shell in args.shell:
            if not shutil.which(shell):
                raise RuntimeError(f'Required test shell is missing: {shell}')
            for line, expected in CASES:
                if shell == 'fish':
                    check_fish(env, directory, line, expected)
                else:
                    check_interactive(shell, env, directory, line, expected)
                print(f'{shell}: {line!r} -> {", ".join(expected)}', flush=True)


if __name__ == '__main__':
    main()
