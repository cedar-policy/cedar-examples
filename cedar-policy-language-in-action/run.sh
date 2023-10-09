#!/bin/bash

rc=0

passed() {
    local message=$1
    echo "  ✅ PASS: $message"
}

failed() {
    local message=$1
    echo "  ❌ FAIL: $message"
    rc=1
}

validate() {
    local folder=$1
    local policies=$2
    local schema=$3
    echo " Running validation on ${policies}"
    res=$(cedar validate --policies $folder/$policies --schema $folder/$schema)
    if [[ $? == 0 ]]
    then
        passed "validate succeeded"
    else
        failed "validate on ${policies} with result: ${res}"
    fi
}

authorize() {
    local folder=$1
    local policies=$2
    local entities=$3  
    echo " Running authorization on ${policies}"
    for decision in ALLOW DENY
    do
        for file in $folder/$decision/*.json
        do
            IFS=$'\n' read -r -d '' -a tmp_array < <(cedar authorize --policies $folder/$policies  --entities $folder/$entities --request-json "$file" -v && printf '\0')
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

# PhotoApp
echo -e "\nTesting PhotoApp..."
validate "PhotoApp" "photoapp.cedar" "photoapp.cedarschema.json"
authorize "PhotoApp" "photoapp.cedar" "photoapp.cedarentities.json"

# GitApp
echo -e "\nTesting GitApp..."
validate "GitApp" "gitapp.cedar" "gitapp.cedarschema.json"
authorize "GitApp" "gitapp.cedar" "gitapp.cedarentities.json"

exit $rc