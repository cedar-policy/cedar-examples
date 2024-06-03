#!/bin/bash

source ../test_utils.sh

# PhotoApp
echo -e "\nTesting PhotoApp..."
validate "PhotoApp" "photoapp.cedar" "photoapp.cedarschema"
authorize "PhotoApp" "photoapp.cedar" "photoapp.cedarentities.json"
format "PhotoApp" "photoapp.cedar"

# GitApp
echo -e "\nTesting GitApp..."
validate "GitApp" "gitapp.cedar" "gitapp.cedarschema"
authorize "GitApp" "gitapp.cedar" "gitapp.cedarentities.json"
format "GitApp" "gitapp.cedar"

exit "$any_failed"
