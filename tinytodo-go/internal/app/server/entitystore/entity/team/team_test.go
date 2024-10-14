package team

import (
	"encoding/json"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entity"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"testing"
)

func TestTeam(t *testing.T) {
	t.Run("check interface", func(t *testing.T) {
		var e entity.Entity
		team := New(
			TeamUID{
				EntityUID: entityuid.New(entitytype.Team, "temp"),
			},
			nil,
		)
		e = team
		assert.NotNil(t, e)
	})
}

func TestTeamUID_Marshal(t *testing.T) {
	t.Run("marshal TeamUID", func(t *testing.T) {
		teamUID := TeamUID{
			EntityUID: entityuid.New(entitytype.Team, "temp"),
		}
		got, err := json.MarshalIndent(teamUID, "", "  ")
		require.NoError(t, err)

		assert.NotEmpty(t, got)
	})
}
func TestTeam_Unmarshal(t *testing.T) {
	t.Run("unmarshal Team", func(t *testing.T) {
		marshalled := []byte(`
{
  "uid": "Team::\"temp\"",
  "parents": [
	"Application::\"TinyTodo\""
  ]
}
`)
		var team Team
		require.NoError(t, json.Unmarshal(marshalled, &team))
		assert.Equal(
			t,
			TeamUID{
				EntityUID: entityuid.New(entitytype.Team, "temp"),
			},
			team.UID,
		)
		assert.Contains(
			t,
			team.Parents,
			entityuid.New(entitytype.Application, "TinyTodo"),
		)
	})
}
