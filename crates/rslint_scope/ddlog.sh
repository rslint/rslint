#!/bin/sh

YELLOW="\033[1;33m"
GREEN="\033[0;32m"
RED="\033[0;31m"
# Remove coloring
NO_COLOR="\033[0m"

# ddlog directories
DDLOG_INPUT_FILE="ddlog/rslint_scoping.dl"
DDLOG_LIBRARY_DIR="ddlog"
DDLOG_OUTPUT_DIR="."

error() {
    local message="$1"

    if [ "$no_color" = "true" ]; then
        printf "error: $message"
    else
        printf "${RED}error:${NO_COLOR} $message"
    fi
}

warn() {
    local message="$1"

    if [ "$no_color" = "true" ]; then
        printf "warning: $message"
    else
        printf "${YELLOW}warning:${NO_COLOR} $message"
    fi
}

success() {
    local message="$1"

    if [ "$no_color" = "true" ]; then
        printf "$message"
    else
        printf "${GREEN}${message}${NO_COLOR}"
    fi
}

failure() {
    local message="$1"

    if [ "$no_color" = "true" ]; then
        printf "$message"
    else
        printf "${RED}${message}${NO_COLOR}"
    fi
}

check_subcommand() {
    local cmd_name="$1"

    if [ ! -z "$subcommand" ]; then
        error "the subcommand '$cmd_name' was given twice\n"
        exit 101
    fi
}

check_undeclared() {
    local var="$1"
    local arg_name="$2"

    if [ ! -z $var ]; then
        warn "the flag '$arg_name' was given twice\n"
    fi
}

extra_args=""
for arg in "$@"; do
    if [ "$arg" = "--debug" ]; then
        check_undeclared "$debug_flag" "--debug"
        extra_args="$extra_args --output-internal-relations --output-input-relations=INPUT_"
        debug_flag="true"

    elif [ "$arg" = "--no-color" ]; then
        check_undeclared "$no_color" "--no-color"
        no_color="true"

    elif [ "$arg" = "--no-check" ]; then
        check_undeclared "$no_check" "--no-check"
        no_check="true"

    elif [ "$arg" = "--no-fmt" ]; then
        check_undeclared "$no_rustfmt" "--no-fmt"
        extra_args="$extra_args --run-rustfmt"
        no_rustfmt="true"

    elif [ "$arg" = "-o" ] || [ "$arg" = "--output-dir" ]; then
        check_undeclared "$output_dir" "--output-dir"
        output_dir="true"
    
    elif [ "$arg" = "-i" ] || [ "$arg" = "--input-file" ]; then
        check_undeclared "$input_file" "--input-file"
        input_file="true"
    
    elif [ "$arg" = "-L" ] || [ "$arg" = "--library-dir" ]; then
        check_undeclared "$library_dir" "--library-dir"
        library_dir="true"

    elif [ "$arg" = "--no-xtask" ]; then
        check_undeclared "$no_xtask" "--no-xtask"
        no_xtask="true"

    elif [ "$arg" = "--help" ] || [ "$arg" = "-h" ]; then
        printf "USAGE:\n"
        printf "    ./ddlog.sh [SUBCOMMAND] [FLAGS]\n"
        printf "\n"
        printf "FLAGS:\n"
        printf "    -h, --help              Display this help message\n"
        printf "        --no-check          Don't run 'cargo check' on generated code\n"
        printf "        --debug             Enable debug mode (causes ddlog to dump internal tables)\n"
        printf "        --no-color          Disable terminal coloring\n"
        printf "        --no-fmt            Don't run rustfmt\n"
        printf "        --no-xtask          Don't run the xtask datalog procedure\n"
        # TODO: Finish these (need arg passing)
        # printf "    -o, --output-dir        Set the output dir, defaults to $DDLOG_OUTPUT_DIR\n"
        # printf "    -i, --input-file        Set the input file, defaults to $DDLOG_INPUT_FILE\n"
        # printf "    -L, --library-dir       Set extra library search paths, defaults to $DDLOG_LIBRARY_DIR\n"
        printf "\n"
        printf "SUBCOMMANDS (defaults to 'compile'):\n"
        printf "    compile     Compile ddlog source into rust\n"
        printf "    check       Check that the ddlog source is valid\n"
        printf "\n"

        exit 0
    
    elif [ "$arg" = "compile" ]; then
        check_subcommand "compile"
        subcommand="compile"

    elif [ "$arg" = "check" ]; then
        check_subcommand "check"
        subcommand="check"

    elif [ ! -z "$arg" ]; then
        error "unrecognized flag '$arg'\n"
        exit 101
    fi
done

if [ "$subcommand" = "check" ]; then
    if [ "$debug_flag" = "true" ]; then
        warn "'--debug' does nothing in check mode\n"
    fi

    printf "checking "
    compile_action="validate"

elif [ "$subcommand" = "compile" ] || [ -z "$subcommand" ]; then
    printf "compiling "
    compile_action="compile"

else
    error "unrecognized subcommand '$subcommand'\n"
    exit 101
fi

if [ "$debug_flag" = "true" ] && [ "$subcommand" != "check" ]; then
    printf "ddlog in debug mode... "
else
    printf "ddlog... "
fi

ddlog -i $DDLOG_INPUT_FILE \
      -L $DDLOG_LIBRARY_DIR \
      --action $compile_action \
      --output-dir=$DDLOG_OUTPUT_DIR \
      --omit-profile \
      --omit-workspace \
      $extra_args

exit_code=$?
if [ $exit_code -ne 0 ]; then
    failure "failed\n"
    exit $exit_code
else
    success "ok\n"
fi

if [ "$subcommand" = "compile" ] || [ -z "$subcommand" ]; then
    printf "replacing old generated code...\n"
    rm -rf generated
    mv rslint_scoping_ddlog generated
fi

if ( [ "$subcommand" = "compile" ] || [ -z "$subcommand" ] ) && [ "$no_xtask" != "true" ]; then
    printf "running xtask code cleanup...\n"
    cargo --quiet run --package xtask --bin xtask -- datalog

    exit_code=$?
    if [ $exit_code -ne 0 ]; then
        printf "failed xtask cleanup\n"
    else
        printf "finised xtask cleanup\n"
    fi
fi

if ( [ "$subcommand" = "compile" ] || [ -z "$subcommand" ] ) && [ "$no_check" != "true" ]; then
    cd generated

    printf "checking generated code... "
    cargo --quiet check

    exit_code=$?
    if [ $exit_code -ne 0 ]; then
        failure "failed\n"
        exit $exit_code
    else
        success "ok\n"
    fi
fi

if [ "$subcommand" = "compile" ] || [ -z "$subcommand" ]; then
    command_name="compiling"
elif [ "$subcommand" = "check" ]; then
    command_name="checking"
fi
printf "finished %s ddlog\n" "$command_name"
