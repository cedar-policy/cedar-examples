package server

import (
	"context"
	"fmt"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/action"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"github.com/cedar-policy/cedar-go"
	"log/slog"
)

// isAuthorized relies on the Cedar policy engine to check if a principal can perform an action on a resource.
//
// Currently, contexts are not supported.
//
// Non-existent entities (resources) will result in an error. (TODO: we may not want this behaviour)
func (s *Server) isAuthorized(
	ctx context.Context,
	principal entityuid.EntityUID,
	action action.Action,
	resource entityuid.EntityUID,
) (bool, cedar.Diagnostic, error) {

	// we have to generate entities every time, because the entities may have been updated
	entities, err := s.es.AsEntities()
	if err != nil {
		return false, cedar.Diagnostic{}, fmt.Errorf("failed to convert entities: %w", err)
	}

	decision, diagnostic := s.ps.IsAuthorized(
		entities,
		cedar.Request{
			Principal: principal.EntityUID,
			Action:    action.GetEUID().EntityUID,
			Resource:  resource.EntityUID,
			Context:   nil,
		},
	)

	s.logger.InfoContext(
		ctx,
		"processed authorization request",
		slog.Any("decision", decision),
		slog.Any("diagnostic", diagnostic),
	)

	return bool(decision), diagnostic, nil
}
