package task

import (
	"encoding/json"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entity"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/taskstate"
	"github.com/cedar-policy/cedar-go/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"testing"
)

func TestTask(t *testing.T) {
	t.Run("check interface", func(t *testing.T) {
		var e entity.Entity
		task := &Task{
			UID: TaskUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.Task.String()),
						ID:   "1",
					},
				},
			},
			ID:    0,
			Name:  "Do something",
			State: 0,
		}
		e = task
		assert.NotNil(t, e)
	})
}

func TestTask_Marshal(t *testing.T) {
	t.Run("check marshal valid case", func(t *testing.T) {
		task := Task{
			UID: TaskUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.Task.String()),
						ID:   "1",
					},
				},
			},
			ID:    1,
			Name:  "Write App",
			State: taskstate.Checked,
		}

		got, err := json.MarshalIndent(task, "", "  ")
		require.NoError(t, err)

		assert.NotEmpty(t, got)
	})
}
