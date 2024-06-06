#!/bin/bash

source ../test_utils.sh

# GitHub
echo -e "\nTesting github_example..."
validate "github_example" "policies.cedar" "policies.cedarschema"
authorize "github_example" "policies.cedar" "entities.json"
format "github_example" "policies.cedar"

# Document cloud
echo -e "\nTesting document_cloud..."
validate "document_cloud" "policies.cedar" "policies.cedarschema"
authorize "document_cloud" "policies.cedar" "entities.json"
format "document_cloud" "policies.cedar"

# Tags & roles
echo -e "\nTesting Tags & Roles..."
validate "tags_n_roles" "policies.cedar" "policies.cedarschema"
authorize "tags_n_roles" "policies.cedar" "entities.json" "policies.cedarschema"
format "tags_n_roles" "policies.cedar"

# Hotel chains
echo -e "\nTesting Hotels (static)..."
validate "hotel_chains/static" "policies.cedar" "policies.cedarschema"
authorize "hotel_chains/static" "policies.cedar" "entities.json" "policies.cedarschema"
#format "hotel_chains/static" "policies.cedar"

# Hotel chains
echo -e "\nTesting Hotels (templated)..."
validate "hotel_chains/templated" "policies.cedar" "policies.cedarschema" "linked"
authorize "hotel_chains/templated" "policies.cedar" "entities.json" "policies.cedarschema" "linked"
#format "hotel_chains/static" "policies.cedar"

exit "$any_failed"
