#!/bin/bash

source ../test_utils.sh

# PhotoApp
echo -e "\nTesting PhotoApp..."
validate "PhotoApp" "photoapp.cedar" "photoapp.cedarschema.json"
authorize "PhotoApp" "photoapp.cedar" "photoapp.cedarentities.json"

# GitApp
echo -e "\nTesting GitApp..."
validate "GitApp" "gitapp.cedar" "gitapp.cedarschema.json"
authorize "GitApp" "gitapp.cedar" "gitapp.cedarentities.json"

exit "$ANY_FAILED"
