// Package action contains the enum Action that represents the different actions supported by entitystore.EntityStore.
package action

import (
	"code.byted.org/binaryauthorization/tinytodo-go/internal/app/server/entitystore"
	"code.byted.org/binaryauthorization/tinytodo-go/internal/app/server/entitystore/entitytype"
	"strings"
)

// Action is an enum that represents the different entity types supported by Cedar.
type Action int

const (
	Unknown Action = iota
	EditShare
	UpdateTask
	CreateTask
	DeleteTask
	GetLists
	GetList
	CreateList
	UpdateList
	DeleteList
)

var (
	Name = map[Action]string{
		Unknown:    "Action::\"Unknown\"",
		EditShare:  "Action::\"EditShare\"",
		UpdateTask: "Action::\"UpdateTask\"",
		CreateTask: "Action::\"CreateTask\"",
		DeleteTask: "Action::\"DeleteTask\"",
		GetLists:   "Action::\"GetLists\"",
		GetList:    "Action::\"GetList\"",
		CreateList: "Action::\"CreateList\"",
		UpdateList: "Action::\"UpdateList\"",
		DeleteList: "Action::\"DeleteList\"",
	}

	EntityUID = map[Action]entitystore.EntityUID{}
)

func init() {
	// verify that all Actions are valid EUIDs
	for k, act := range Name {
		euid, err := entitystore.ParseEntityUID(act)
		if err != nil {
			panic(err)
		}
		if euid.Type != entitytype.Action.String() {
			panic(err)
		}
		EntityUID[k] = euid
	}
}

func (a Action) String() string {
	return Name[a]
}

// Parse parses a given string as EntityType using strings.EqualFold() (case insensitive)
func Parse(act string) Action {
	for res := range Name {
		if strings.EqualFold(res.String(), act) {
			return res
		}
	}
	return Unknown
}

func (a Action) GetEUID() entitystore.EntityUID {
	return EntityUID[a]
}
