package entitystore

import (
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"github.com/cedar-policy/cedar-go/types"
)

// UserUID is a transparent wrapper around EntityUID, to make it clear that we want a User's EntityUID.
//
// Note that we use inheritance instead of alias, because we want to inherit methods. See [blog post].
//
// [blog post]: https://sentry.io/answers/alias-type-definitions/
type UserUID struct {
	entityuid.EntityUID
}

// User represents the user entity.
type User struct {
	EUID     UserUID               `json:"euid"` // note the spelling
	Location string                `json:"location"`
	JobLevel int                   `json:"joblevel"`
	Parents  []entityuid.EntityUID `json:"parents"`
}

// NewUser creates a new User; if parents is nil, we create an empty slice so that there will be no problems with
// client processing.
func NewUser(uid UserUID, location string, jobLevel int, parents []entityuid.EntityUID) *User {
	if parents == nil {
		parents = []entityuid.EntityUID{}
	}
	return &User{
		EUID:     uid,
		Location: location,
		JobLevel: jobLevel,
		Parents:  parents,
	}
}

// AsCedarEntity converts User into a types.Entity, to be passed to the Cedar authorization engine when it evaluates a
// request.
func (u *User) AsCedarEntity() *types.Entity {
	var parents []types.EntityUID
	for _, parent := range u.Parents {
		parents = append(parents, parent.EntityUID)
	}
	return &types.Entity{
		UID:     u.EUID.EntityUID.EntityUID,
		Parents: parents,
		Attributes: types.Record{
			"location": types.String(u.Location),
			"joblevel": types.Long(u.JobLevel),
		},
	}
}
