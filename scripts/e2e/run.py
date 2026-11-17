#!/usr/bin/env python3
"""Lance le scénario e2e natif d'Abcom sur l'hôte macOS et sur les profils du laboratoire local, une VM à la fois.

Le laboratoire (Test-local) n'est pas copié dans le dépôt : il est appelé par son chemin absolu, donné par
`--lab` ou par la variable `ABCOM_LAB`. Les binaires viennent de la CI (`gh run download -p 'e2e-*'`) :
rien n'est compilé dans les VM.
"""
import argparse
import datetime
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import zipfile

HERE = Path(__file__).resolve().parent
ALL_PROFILES = ['host', 'linux-gnome', 'linux-gnome-tray', 'linux-kde', 'linux-xfce', 'windows']
BINARIES = {'host': 'abcom-macos-aarch64', 'linux': 'abcom-linux-aarch64', 'windows': 'abcom-windows-x86_64.exe'}
LINUX_DIR = '/tmp/abcom-e2e'
WINDOWS_DIR = r'C:\ProgramData\abcom-e2e'


def find_binary(artifacts, name):
    matches = sorted(Path(artifacts).rglob(name))
    if not matches:
        raise SystemExit(f'Binaire introuvable dans {artifacts} : {name}')
    return matches[0]


def lab(lab_py, *args, check=True):
    print('$ lab.py ' + ' '.join(args[:3]) + (' …' if len(args) > 3 else ''), flush=True)
    result = subprocess.run([sys.executable, str(lab_py), *args], text=True, capture_output=True)
    if check and result.returncode:
        raise RuntimeError(f'lab.py {args[0]} a échoué ({result.returncode}) : {result.stderr.strip()}')
    return result


def run_host(artifacts, out):
    binary = find_binary(artifacts, BINARIES['host'])
    binary.chmod(0o755)
    result = subprocess.run(['bash', str(HERE / 'scenario_unix.sh'), str(binary), str(out)], text=True, capture_output=True)
    return result.returncode, result.stdout + result.stderr


def run_linux(lab_py, profile, artifacts, out):
    binary = find_binary(artifacts, BINARIES['linux'])
    lab(lab_py, 'exec', profile, '--', '/bin/bash', '-c', f'rm -rf {LINUX_DIR} && mkdir -p {LINUX_DIR} && chmod 777 {LINUX_DIR}')
    lab(lab_py, 'push', profile, str(binary), f'{LINUX_DIR}/abcom')
    lab(lab_py, 'push', profile, str(HERE / 'scenario_unix.sh'), f'{LINUX_DIR}/scenario.sh')
    lab(lab_py, 'exec', profile, '--', '/bin/chmod', '755', f'{LINUX_DIR}/abcom', f'{LINUX_DIR}/scenario.sh')
    result = lab(lab_py, 'exec', profile, '--as-user', '--', '/bin/bash', f'{LINUX_DIR}/scenario.sh',
                 f'{LINUX_DIR}/abcom', f'{LINUX_DIR}/out', check=False)
    lab(lab_py, 'exec', profile, '--', '/bin/tar', '-czf', f'{LINUX_DIR}/out.tgz', '-C', f'{LINUX_DIR}/out', '.')
    archive = out / 'out.tgz'
    lab(lab_py, 'pull', profile, f'{LINUX_DIR}/out.tgz', str(archive))
    with tarfile.open(archive) as tar:
        tar.extractall(out, filter='data')
    archive.unlink()
    return result.returncode, result.stdout + result.stderr


def run_windows(lab_py, profile, artifacts, out):
    binary = find_binary(artifacts, BINARIES['windows'])
    ps = 'powershell.exe'
    lab(lab_py, 'exec', profile, '--', ps, '-NoProfile', '-Command',
        f"Remove-Item '{WINDOWS_DIR}' -Recurse -Force -ErrorAction SilentlyContinue; "
        f"New-Item -ItemType Directory '{WINDOWS_DIR}' | Out-Null; icacls '{WINDOWS_DIR}' /grant '*S-1-5-32-545:(OI)(CI)M' | Out-Null")
    lab(lab_py, 'push', profile, str(binary), WINDOWS_DIR + r'\abcom.exe')
    lab(lab_py, 'push', profile, str(HERE / 'scenario_windows.ps1'), WINDOWS_DIR + r'\scenario.ps1')
    result = lab(lab_py, 'exec', profile, '--as-user', '--', ps, '-NoProfile', '-ExecutionPolicy', 'Bypass',
                 '-File', WINDOWS_DIR + r'\scenario.ps1', '-Binary', WINDOWS_DIR + r'\abcom.exe',
                 '-Out', WINDOWS_DIR + r'\out', check=False)
    lab(lab_py, 'exec', profile, '--', ps, '-NoProfile', '-Command',
        f"Compress-Archive -Path '{WINDOWS_DIR}\\out\\*' -DestinationPath '{WINDOWS_DIR}\\out.zip' -Force")
    archive = out / 'out.zip'
    lab(lab_py, 'pull', profile, WINDOWS_DIR + r'\out.zip', str(archive))
    with zipfile.ZipFile(archive) as bundle:
        bundle.extractall(out)
    archive.unlink()
    return result.returncode, result.stdout + result.stderr


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('profiles', nargs='+', help='profils (' + ', '.join(ALL_PROFILES) + ') ou « all »')
    parser.add_argument('--lab', default=os.environ.get('ABCOM_LAB'), help='chemin absolu de lab.py (ou ABCOM_LAB)')
    parser.add_argument('--artifacts', required=True, help='dossier des binaires e2e téléchargés depuis la CI')
    parser.add_argument('--out', help='dossier de résultats (défaut : results/ du laboratoire)')
    args = parser.parse_args()

    profiles = ALL_PROFILES if args.profiles == ['all'] else args.profiles
    unknown = [p for p in profiles if p not in ALL_PROFILES]
    if unknown:
        parser.error('profils inconnus : ' + ', '.join(unknown))
    needs_lab = any(p != 'host' for p in profiles)
    lab_py = Path(args.lab).expanduser().resolve() if args.lab else None
    if needs_lab and (lab_py is None or not lab_py.is_file()):
        parser.error('lab.py introuvable : donner --lab ou ABCOM_LAB')

    stamp = datetime.datetime.now().strftime('%Y%m%d-%H%M%S')
    root = Path(args.out) if args.out else (lab_py.parent / 'results' if lab_py else Path('target')) / f'abcom-e2e-{stamp}'
    summary = {}
    for profile in profiles:
        out = root / profile
        out.mkdir(parents=True, exist_ok=True)
        print(f'=== {profile}', flush=True)
        started = False
        try:
            if profile == 'host':
                code, log = run_host(args.artifacts, out)
            else:
                lab(lab_py, 'start', profile)
                started = True
                runner = run_windows if profile == 'windows' else run_linux
                code, log = runner(lab_py, profile, args.artifacts, out)
        except (RuntimeError, OSError) as error:
            code, log = 1, f'ERREUR {error}\n'
        finally:
            if started:
                lab(lab_py, 'reset', profile, check=False)
        (out / 'scenario.log').write_text(log)
        print(log, end='' if log.endswith('\n') else '\n', flush=True)
        summary[profile] = 'réussi' if code == 0 else f'échec ({code})'

    (root / 'summary.json').write_text(json.dumps(summary, indent=2, ensure_ascii=False) + '\n')
    print('Résumé : ' + json.dumps(summary, ensure_ascii=False))
    print('Résultats : ' + str(root))
    return 0 if all(value == 'réussi' for value in summary.values()) else 1


if __name__ == '__main__':
    sys.exit(main())
