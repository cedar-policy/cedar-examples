#!/bin/bash

# Usage: `source` this script from a testing script in a subdirectory. This
# will make the `validate` and `authorize` functions available. Use these to
# test that polices validate and authorizer as expected. The env var
# `ANY_FAILED` is exported as `0` when this script is sourced. A failing test
# case causes the `ANY_FAILED` env var to be set to `1`. A passing test does
# not change the variable. After running all tests `exit "$ANY_FAILED"` to exit
# non-zero for any failing test case.

passed() {
    local message=$1
    echo "  ✅ PASS: $message"
}

export ANY_FAILED=0
failed() {
    local message=$1
    echo "  ❌ FAIL: $message"
    export ANY_FAILED=1
}


# Call this function to assert that policies in the directory `$1/$2` validate
# with the schema `$1/$3`. Set `ANY_FAILED` env var to `1` if the policy does
# not validate.
validate() {
    local folder=$1
    local policies=$2
    local schema=$3
    echo " Running validation on ${policies}"
    res="$(cedar validate --policies "$folder/$policies" --schema "$folder/$schema")"
    if [[ $? == 0 ]]
    then
        passed "validate succeeded"
    else
        failed "validate on ${policies} with result: ${res}"
    fi
}

# Call this function to assert that authorization requests defined in
# `$1/ALLOW/*.json` and `$1/DENY/*.json` evaluate with expected authorization
# result given policies in directory `$1/$2` and entities in `$1/$3`.
authorize() {
    local folder=$1
    local policies=$2
    local entities=$3
    echo " Running authorization on ${policies}"
    for decision in ALLOW DENY
    do
        for file in "$folder/$decision"/*.json
        do
            IFS=$'\n' read -r -d '' -a tmp_array < <(cedar authorize --policies "$folder/$policies"  --entities "$folder/$entities" --request-json "$file" -v && printf '\0')
            res="${tmp_array[0]}"
            unset tmp_array[0]
            unset tmp_array[1]
            policyIds="$(IFS=\;; echo "${tmp_array[*]}")"
            jsonfile="$(echo "$file" | cut -d '/' -f 3)"
            if [ "$res" != "$decision" ]
            then
                failed "decision \"${res}\" for ${jsonfile}, expected \"${decision}\""
            else
                passed "decision \"${decision}\" for ${jsonfile} determined by policy id(s):${policyIds}"
            fi
        done
    done
}

echo "Using $(cedar --version)"
