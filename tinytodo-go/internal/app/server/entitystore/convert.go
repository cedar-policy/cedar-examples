package entitystore

import (
	"github.com/cedar-policy/cedar-go/types"
)

// AsEntities converts EntityStore's native objects into types.Entities, to be passed to the Cedar authorization engine
// when it evaluates a request.
func (e *EntityStore) AsEntities() (types.Entities, error) {

	es := make(types.Entities)

	// process users

	for _, user := range e.Users {
		es[user.EUID.EntityUID.EntityUID] = user.AsCedarEntity()
	}

	// process teams

	for _, team := range e.Teams {
		es[team.UID.EntityUID.EntityUID] = team.AsCedarEntity()
	}

	// process lists

	for _, list := range e.Lists {
		es[list.UID.EntityUID.EntityUID] = list.AsCedarEntity()
	}

	// process application

	es[e.App.EUID.EntityUID] = e.App.AsCedarEntity()

	return es, nil
}
