package entitystore

import (
	"encoding/json"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entity/team"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entity/user"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"os"
	"path"
	"testing"
)

func readFile(t *testing.T, prefix, filename string) []byte {
	t.Helper()
	res, err := os.ReadFile(path.Join(prefix, filename))
	require.NoError(t, err)
	return res
}

func Test_EntityStore(t *testing.T) {
	t.Run("basic", func(t *testing.T) {
		f := readFile(t, "../../../../", "entities.json")
		var es EntityStore
		require.NoError(t, json.Unmarshal(f, &es))

		userUID := user.UserUID{
			EntityUID: entityuid.New(entitytype.User, "kesha"),
		}
		teamUID := team.TeamUID{
			EntityUID: entityuid.New(entitytype.Team, "temp"),
		}
		applicationEUID := entityuid.New(entitytype.Application, "TinyTodo")

		assert.Contains(t, es.Users, userUID)
		assert.Equal(
			t,
			user.New(
				userUID,
				"ABC17",
				5,
				[]entityuid.EntityUID{
					// order matters
					applicationEUID,
					teamUID.EntityUID,
				},
			),
			es.Users[userUID],
		)

		assert.Contains(t, es.Teams, teamUID)
		assert.Equal(
			t,
			team.New(teamUID, []entityuid.EntityUID{applicationEUID}),
			es.Teams[teamUID],
		)
		assert.Equal(t, applicationEUID, es.App.EUID)
	})
}
