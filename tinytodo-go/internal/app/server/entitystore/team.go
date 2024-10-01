package entitystore

import (
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"github.com/cedar-policy/cedar-go/types"
)

// TeamUID is a transparent wrapper around EntityUID, to make it clear that we want a Team's EntityUID.
//
// Note that we use inheritance instead of alias, because we want to inherit methods. See [blog post].
//
// [blog post]: https://sentry.io/answers/alias-type-definitions/
type TeamUID struct {
	entityuid.EntityUID
}

// Team represents the team entity.
//
// In TinyTodo, List.UID.EntityUID.EntityUID.ID is constrained to be string-formatted non-negative integers.
// For example, "0", "1", ...
//
// This is because List should only be created via the APIs, hence the generation of the ID is controlled.
type Team struct {
	UID     TeamUID               `json:"uid"`     // note the naming
	Parents []entityuid.EntityUID `json:"parents"` // can be TeamUID or UserUID
}

// NewTeam creates a new Team; if parents is nil, we create an empty slice so that there will be no problems with
// client processing.
func NewTeam(uid TeamUID, parents []entityuid.EntityUID) *Team {
	if parents == nil {
		parents = []entityuid.EntityUID{}
	}
	return &Team{uid, parents}
}

// AsCedarEntity converts Team into a types.Entity, to be passed to the Cedar authorization engine when it evaluates a
// request.
func (t *Team) AsCedarEntity() *types.Entity {
	var parents []types.EntityUID
	for _, parent := range t.Parents {
		parents = append(parents, parent.EntityUID)
	}
	return &types.Entity{
		UID:     t.UID.EntityUID.EntityUID,
		Parents: parents,
	}
}
