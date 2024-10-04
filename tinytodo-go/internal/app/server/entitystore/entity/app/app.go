package app

import (
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"github.com/cedar-policy/cedar-go/types"
)

// App represents the application entity (in this case, TinyTodo).
type App struct {
	EUID entityuid.EntityUID `json:"euid"`
}

// AsCedarEntity converts App into a types.Entity, to be passed to the Cedar authorization engine when it evaluates a
// request.
func (a *App) AsCedarEntity() *types.Entity {
	return &types.Entity{
		UID: a.EUID.EntityUID,
		//Parents:    nil,
		//Attributes: nil,
	}
}
