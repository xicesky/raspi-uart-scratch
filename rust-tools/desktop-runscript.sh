#!/bin/bash

# shellcheck disable=SC2034,SC2164
THIS_SCRIPT_DIR="$(cd "$(dirname "$0")"; pwd)"

NM_CONNECTION_NAME="${NM_CONNECTION_NAME:-barki}"

# Stores the last command executed by invoke()
declare -a LAST_COMMAND
LAST_COMMAND=()

# ... and it's exit code
declare -g LAST_COMMAND_EXITCODE
LAST_COMMAND_EXITCODE=0

declare -g darkblue darkgrey red green blue normal
if ! which tput >/dev/null 2>&1 ; then
    tput() {
        return 0
    }
fi

darkblue="$(tput setaf 4)"
darkgrey="$(tput setaf 8)"
red="$(tput setaf 9)"
green="$(tput setaf 10)"
blue="$(tput setaf 12)"
normal="$(tput sgr0)"
spacer="$(printf '=%.0s' {1..100})"

invoke() {
    LAST_COMMAND=("$@")
    echo "${blue}$(printf "%q " "$@")${normal}" 1>&2
    LAST_COMMAND_EXITCODE=0
    "$@"
    LAST_COMMAND_EXITCODE="$?"
    return $LAST_COMMAND_EXITCODE
}

report_state() {
    echo "$darkgrey$spacer$normal"
    echo "${green}$1${normal}"
    if [[ -n "$2" ]] ; then
        echo "$blue$2$normal"
    fi
    echo ""
}

report_command_failure() {
    if [[ "$LAST_COMMAND_EXITCODE" -ne 0 ]] ; then
        echo "${red}Last command executed:"
        echo "    $(printf "%q " "${LAST_COMMAND[@]}")"
        echo "Returned exit code ${LAST_COMMAND_EXITCODE}"
        echo "${normal}"
    fi
}

err_exit() {
    local rc="$1"; shift
    echo "$*"
    report_command_failure 1>&2
    return "$rc"
}

network_connected() {
    LC_ALL=C invoke nmcli -f GENERAL.STATE con show "$NM_CONNECTION_NAME" | grep -q -E '\bactiv'
}

update_from_github() {
    invoke git checkout main || { err_exit 1 "Fehler: Kann nicht auf branch \"main\" wechseln." || return $?; }
    invoke git fetch origin || { err_exit 1 "Fehler: Kann Quellcode nicht von github laden." || return $?; }
    invoke git reset --hard origin/main || { err_exit 1 "Fehler: Kann Quellcode nicht zurücksetzen." || return $?; }
}

run_rust_program() {
    local git_hash git_hash_short
    git_hash_short="$(git rev-parse --short HEAD)"
    git_hash="$(git rev-parse HEAD)"

    report_state "Compiliere Version ${git_hash_short}" "${blue}(Volle Versionsnummer: $git_hash)${normal}" 1>&2
    invoke cargo build || { err_exit 1 "Fehler beim Compilieren des Quellcodes." || return $?; }
    invoke cargo test || { err_exit 1 "Fehler beim Programm-Selbsttest." || return $?; }

    report_state "Führe Programm aus..." 1>&2
    invoke cargo run
    report_command_failure
}

main() {
    report_state "Lade neuen Quellcode..." 1>&2
    if ! network_connected ; then
        echo "Netzwerk (\"$NM_CONNECTION_NAME\") nicht verbunden!" 1>&2
        return 1
    fi
    invoke cd "$THIS_SCRIPT_DIR" || { err_exit 1 "Fehler: Konnte nicht ins Quellcodeverzeichnis wechseln." || return $?; }
    update_from_github || return $?

    run_rust_program || return $?
}

main "$@"
