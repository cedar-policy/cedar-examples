package entitystore

import (
	"code.byted.org/binaryauthorization/tinytodo-go/internal/app/server/entitystore/entitytype"
	"encoding/json"
	"github.com/cedar-policy/cedar-go"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"testing"
)

func TestList(t *testing.T) {
	t.Run("check interface", func(t *testing.T) {
		var e Entity
		l := NewList(
			ListUID{
				EntityUID: EntityUID{
					EntityUID: cedar.EntityUID{
						Type: entitytype.List.String(),
						ID:   "1",
					},
				},
			},
			"Cedar blog post",
			UserUID{
				EntityUID: EntityUID{
					EntityUID: cedar.EntityUID{
						Type: entitytype.User.String(),
						ID:   "kesha",
					},
				},
			},
			TeamUID{
				EntityUID: EntityUID{
					EntityUID: cedar.EntityUID{
						Type: entitytype.Team.String(),
						ID:   "temp",
					},
				},
			},
			TeamUID{
				EntityUID: EntityUID{
					EntityUID: cedar.EntityUID{
						Type: entitytype.Team.String(),
						ID:   "admin",
					},
				},
			},
			nil,
		)
		e = l
		require.NotNil(t, e)
	})
}

func TestList_Marshal(t *testing.T) {
	t.Run("check marshal valid case", func(t *testing.T) {
		list := NewList(
			ListUID{
				EntityUID: EntityUID{
					EntityUID: cedar.EntityUID{
						Type: entitytype.List.String(),
						ID:   "1",
					},
				},
			},
			"Cedar blog post",
			UserUID{
				EntityUID: EntityUID{
					EntityUID: cedar.EntityUID{
						Type: entitytype.User.String(),
						ID:   "kesha",
					},
				},
			},
			TeamUID{
				EntityUID: EntityUID{
					EntityUID: cedar.EntityUID{
						Type: entitytype.Team.String(),
						ID:   "temp",
					},
				},
			},
			TeamUID{
				EntityUID: EntityUID{
					EntityUID: cedar.EntityUID{
						Type: entitytype.Team.String(),
						ID:   "admin",
					},
				},
			},
			nil,
		)

		got, err := json.MarshalIndent(list, "", "  ")
		require.NoError(t, err)

		var recovered List
		require.NoError(t, json.Unmarshal(got, &recovered))

		assert.Equal(t, *list, recovered)
	})
}
