package task

import (
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/taskstate"
	"github.com/cedar-policy/cedar-go/types"
)

// TaskUID is a transparent wrapper around EntityUID, to make it clear that we want a Task's EntityUID.
//
// Note that we use inheritance instead of alias, because we want to inherit methods. See [blog post].
//
// [blog post]: https://sentry.io/answers/alias-type-definitions/
type TaskUID struct {
	entityuid.EntityUID
}

// Task represents the task entity.
type Task struct {
	UID   TaskUID             `json:"uid"`
	ID    int                 `json:"id"`
	Name  string              `json:"name"`
	State taskstate.TaskState `json:"state"`
}

// AsCedarEntity converts Task into a types.Entity, to be passed to the Cedar authorization engine when it evaluates a
// request.
func (t *Task) AsCedarEntity() *types.Entity {

	records := make(types.Record)
	records["name"] = types.String(t.Name)
	records["state"] = types.String(t.State.String())

	return &types.Entity{
		UID: t.UID.EntityUID.EntityUID,
		//Parents:    nil,
		Attributes: records,
	}
}
