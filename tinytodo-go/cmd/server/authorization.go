package main

import (
	"code.byted.org/binaryauthorization/tinytodo-go/internal/app/server/entitystore"
	"fmt"
	"github.com/cedar-policy/cedar-go"
	"os"
)

const (
	DefaultCedarPolicyFileName = "policies.cedar"
	DefaultEntitiesFileName    = "entities.json" // this is not in the Cedar entity schema, conversion required
)

func prepareCedarPolicyEntities() (*entitystore.EntityStore, cedar.PolicySet, error) {

	entitiesFile, err := os.ReadFile(DefaultEntitiesFileName)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to read EntitiesFile: %w", err)
	}

	es, err := entitystore.New(entitiesFile)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to create EntityStore: %w", err)
	}

	psFile, err := os.ReadFile(DefaultCedarPolicyFileName)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to read Cedar policy file: %w", err)
	}

	ps, err := cedar.NewPolicySet(DefaultCedarPolicyFileName, psFile)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to create Cedar policy set: %w", err)
	}

	return es, ps, nil
}
