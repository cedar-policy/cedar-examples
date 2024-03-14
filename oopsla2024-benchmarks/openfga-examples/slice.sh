#!/bin/bash

slice() {
    local folder=$1
    echo "${folder}:"
    (
    cd "$folder" || exit
    for file in *_requests/*.json
    do
        printf "  - %s: " "$file"
        if [[ -f "./linked" ]]
        then
            link_arg="--template-linked ./linked"
        fi
        cedar slice --policies ./policies.cedar --entities ./entities.json --request-json "$file" $link_arg
    done
    )
}

folders=("gdrive" "gdrive-templates" "github" "github-templates")
for folder in "${folders[@]}";
do
    slice "$folder"
done
