package entitystore

import (
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/taskstate"
	"github.com/cedar-policy/cedar-go/types"
	"strconv"
)

// ListUID is a transparent wrapper around EntityUID, to make it clear that we want a List's EntityUID.
//
// Note that we use composition instead of alias, because we want to inherit methods. See [blog post].
//
// [blog post]: https://sentry.io/answers/alias-type-definitions/
type ListUID struct {
	entityuid.EntityUID
}

// List represents the list entity.
//
// In TinyTodo, List.UID.EntityUID.EntityUID.ID is constrained to be string-formatted non-negative integers.
// For example, "0", "1", ...
//
// This is because List should only be created via the APIs, hence the generation of the ID is controlled.
type List struct {
	UID     ListUID `json:"uid"`
	Name    string  `json:"name"`
	Owner   UserUID `json:"owner"`
	Readers TeamUID `json:"readers"` // plural because its a team of readers
	Editors TeamUID `json:"editors"` // plural because its a team of editors
	Tasks   []*Task `json:"tasks"`
}

// NewList creates a new List; if tasks is nil, we create an empty slice so that there will be no problems with
// client processing.
func NewList(uid ListUID, name string, owner UserUID, readers TeamUID, editors TeamUID, tasks []*Task) *List {
	if tasks == nil {
		tasks = []*Task{}
	}
	return &List{uid, name, owner, readers, editors, tasks}
}

// AsCedarEntity converts List into a types.Entity, to be passed to the Cedar authorization engine when it evaluates a
// request.
func (l *List) AsCedarEntity() *types.Entity {

	records := make(types.Record)

	// be careful - it is easy to get types.Value wrong

	records["name"] = types.String(l.Name)
	records["owner"] = l.Owner.EntityUID.EntityUID
	records["readers"] = l.Readers.EntityUID.EntityUID
	records["editors"] = l.Editors.EntityUID.EntityUID

	var tasks types.Set
	for _, t := range l.Tasks {
		tasks = append(tasks, t.UID.EntityUID.EntityUID)
	}
	records["tasks"] = tasks // we include tasks because this is what the Rust implementation does

	return &types.Entity{
		UID: l.UID.EntityUID.EntityUID,
		//Parents:    nil,
		Attributes: records,
	}
}

// InsertTask generates a monotonically increasing non-negative integer as the ID for a new task and inserts it to the
// parent List. Returns the ID for the new task.
//
// Although the task ID starts from 0, the client will adjust it with a +1 offset.
func (l *List) InsertTask(name string) int {
	id := len(l.Tasks) // simply use the current number of tasks (non-negative integer) as the ID for the next task
	l.Tasks = append(l.Tasks, &Task{
		UID: TaskUID{
			EntityUID: entityuid.EntityUID{
				EntityUID: types.EntityUID{
					Type: types.EntityType(entitytype.Task.String()),
					ID:   types.String(strconv.Itoa(id)),
				},
			},
		},
		ID:    id,
		Name:  name,
		State: taskstate.Unchecked,
	})
	return id
}

// DeleteTask deletes a Task from the slice List.Tasks and shifts the remaining Task to the left.
//
// The caller has to ensure that the taskIndex
func (l *List) DeleteTask(taskIndex int) {
	l.Tasks = append(l.Tasks[:taskIndex], l.Tasks[taskIndex+1:]...)
}
