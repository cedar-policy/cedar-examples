#!/bin/bash

failed() {
    message=$1
    echo "$message"
    exit 1
}

validate() {
    local folder=$1
    echo "Running validation on ${folder}"
    (
     cd "$folder" || exit
     res=$(cedar validate --policies policies.cedar --schema "$folder".cedarschema.json)
     if [[ $res != "Validation Passed" ]]
     then
         failed "Failed validation on ${folder} with result: ${res}"
     else
         echo "Validation succeeded"
     fi
    )
}

authorize() {
    local folder=$1
    echo "Running authorization on ${folder}"
    verdict=$2
    cd "$folder" || exit
    for file in *.json
    do
        if [[ -f "../linked" ]]
        then
            link_arg="--template-linked ../linked"
        fi
        res=$(cedar authorize --policies ../policies.cedar --entities ../entities.json --request-json "$file" $link_arg | xargs)
        if [[ "$res" != "$verdict"* ]]
        then
            failed "Failed authorization on ${file} with result: ${res}"
        else
            echo "Authorization succeeded on ${file} with expected response \"${verdict}\""
        fi
    done
    cd ../../
}

folders=("gdrive" "gdrive-templates" "github" "github-templates")
for folder in "${folders[@]}";
do
    # validate policies
    validate "$folder"
    # authorize policies
    authorize "$folder""/allow_requests" ALLOW
    authorize "$folder""/deny_requests" "DENY"
done
