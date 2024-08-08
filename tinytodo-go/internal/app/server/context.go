package server

import (
	"code.byted.org/binaryauthorization/tinytodo-go/internal/app/server/entitystore"
	"code.byted.org/binaryauthorization/tinytodo-go/internal/app/server/entitystore/action"
	"errors"
	"fmt"
	"github.com/cedar-policy/cedar-go"
)

// isAuthorized relies on the Cedar policy engine to check if a principal can perform an action on a resource.
//
// Currently, contexts are not supported.
//
// Non-existent entities (resources) will result in an error. (TODO: we may not want this behaviour)
func (s *Server) isAuthorized(
	principal entitystore.EntityUID,
	action action.Action,
	resource entitystore.EntityUID,
) (bool, []string, error) {

	// we have to generate entities every time, because the entities may have been updated
	entities, err := s.es.AsEntities()
	if err != nil {
		return false, nil, fmt.Errorf("failed to convert entities: %w", err)
	}

	ok, diag := s.ps.IsAuthorized(
		entities,
		cedar.Request{
			Principal: principal.EntityUID,
			Action:    action.GetEUID().EntityUID,
			Resource:  resource.EntityUID,
			Context:   nil,
		},
	)

	var reasons []string

	for _, r := range diag.Reasons {
		reasons = append(
			reasons,
			fmt.Sprintf("Policy: %d", r.Policy),
		)
	}

	if len(diag.Errors) > 0 {
		var errs []error
		for _, e := range diag.Errors {
			errs = append(errs, errors.New(e.String()))
		}
		return false, nil, errors.Join(errs...)
	}

	return bool(ok), reasons, nil
}
