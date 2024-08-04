package entitystore

import (
	"code.byted.org/binaryauthorization/tinytodo-go/internal/app/server/entitystore/entitytype"
	"code.byted.org/binaryauthorization/tinytodo-go/internal/app/server/entitystore/taskstate"
	"encoding/json"
	"github.com/cedar-policy/cedar-go"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"testing"
)

func TestTask(t *testing.T) {
	t.Run("check interface", func(t *testing.T) {
		var e Entity
		task := &Task{
			UID: TaskUID{
				EntityUID: EntityUID{
					EntityUID: cedar.EntityUID{
						Type: entitytype.Task.String(),
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
				EntityUID: EntityUID{
					EntityUID: cedar.EntityUID{
						Type: entitytype.Task.String(),
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
