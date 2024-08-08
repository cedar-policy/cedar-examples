package entitystore

import (
	"github.com/cedar-policy/cedar-go"
)

// App represents the application entity (in this case, TinyTodo).
type App struct {
	EUID EntityUID `json:"euid"`
}

// AsCedarEntity converts App into a cedar.Entity, to be passed to the Cedar authorization engine when it evaluates a
// request.
func (a *App) AsCedarEntity() *cedar.Entity {
	return &cedar.Entity{
		UID: a.EUID.EntityUID,
		//Parents:    nil,
		//Attributes: nil,
	}
}
