package entitystore

import (
	"github.com/cedar-policy/cedar-go"
)

// UserUID is a transparent wrapper around EntityUID, to make it clear that we want a User's EntityUID.
//
// Note that we use inheritance instead of alias, because we want to inherit methods. See [blog post].
//
// [blog post]: https://sentry.io/answers/alias-type-definitions/
type UserUID struct {
	EntityUID
}

// User represents the user entity.
type User struct {
	EUID     UserUID     `json:"euid"` // note the spelling
	Location string      `json:"location"`
	JobLevel int         `json:"joblevel"`
	Parents  []EntityUID `json:"parents"`
}

// NewUser creates a new User; if parents is nil, we create an empty slice so that there will be no problems with
// client processing.
func NewUser(uid UserUID, location string, jobLevel int, parents []EntityUID) *User {
	if parents == nil {
		parents = []EntityUID{}
	}
	return &User{
		EUID:     uid,
		Location: location,
		JobLevel: jobLevel,
		Parents:  parents,
	}
}

// AsCedarEntity converts User into a cedar.Entity, to be passed to the Cedar authorization engine when it evaluates a
// request.
func (u *User) AsCedarEntity() *cedar.Entity {
	var parents []cedar.EntityUID
	for _, parent := range u.Parents {
		parents = append(parents, parent.EntityUID)
	}
	return &cedar.Entity{
		UID:     u.EUID.EntityUID.EntityUID,
		Parents: parents,
		Attributes: cedar.Record{
			"location": cedar.String(u.Location),
			"joblevel": cedar.Long(u.JobLevel),
		},
	}
}
