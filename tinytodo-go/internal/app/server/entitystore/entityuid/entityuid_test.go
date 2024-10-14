package entityuid

import (
	"encoding/json"
	"fmt"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/cedar-policy/cedar-go/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"testing"
)

func Test_EntityUID(t *testing.T) {
	t.Run("verify json.Marshaler interface", func(t *testing.T) {
		var m json.Marshaler
		e := New(entitytype.Application, "TinyTodo")
		m = e
		require.NotNil(t, m)
	})
	t.Run("verify json.Unmarshaler interface", func(t *testing.T) {
		var m json.Unmarshaler
		e := New(entitytype.Application, "TinyTodo")
		m = &e
		require.NotNil(t, m)
	})
}

func TestEntityUID_MarshalJSON(t *testing.T) {
	tests := []struct {
		name      string
		entityUID types.EntityUID
		wantErr   assert.ErrorAssertionFunc
	}{
		{
			"valid user",
			types.NewEntityUID(
				types.EntityType(entitytype.User.String()),
				"kesha",
			),
			assert.NoError,
		},
		{
			"valid team",
			types.NewEntityUID(
				types.EntityType(entitytype.Team.String()),
				"temp",
			),
			assert.NoError,
		},
		{
			"valid application",
			types.NewEntityUID(
				types.EntityType(entitytype.Application.String()),
				"TinyTodo",
			),
			assert.NoError,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			e := EntityUID{
				EntityUID: tt.entityUID,
			}
			got, err := e.MarshalJSON()
			if !tt.wantErr(t, err, fmt.Sprintf("MarshalJSON()")) {
				return
			}
			want := []byte(fmt.Sprintf("%q", e.EntityUID.String()))
			assert.Equalf(t, want, got, "MarshalJSON()")

			var gotE EntityUID
			require.NoError(t, json.Unmarshal(got, &gotE))

			assert.Equal(t, gotE, e)
		})
	}
}

func Test_Parse(t *testing.T) {
	tests := []struct {
		name    string
		euid    string
		want    EntityUID
		wantErr assert.ErrorAssertionFunc
	}{
		{
			name: "valid user ID",
			euid: "User::\"kesha\"",
			want: EntityUID{
				types.EntityUID{
					Type: types.EntityType(entitytype.User.String()),
					ID:   "kesha",
				},
			},
			wantErr: assert.NoError,
		},
		{
			name: "valid user ID without quotes",
			euid: "User::kesha",
			want: EntityUID{
				types.EntityUID{
					Type: types.EntityType(entitytype.User.String()),
					ID:   "kesha",
				},
			},
			wantErr: assert.NoError,
		},
		{
			name: "valid team ID",
			euid: "Team::\"temp\"",
			want: EntityUID{
				types.EntityUID{
					Type: types.EntityType(entitytype.Team.String()),
					ID:   "temp",
				},
			},
			wantErr: assert.NoError,
		},
		{
			name: "valid application ID",
			euid: "Application::\"TinyTodo\"",
			want: EntityUID{
				types.EntityUID{
					Type: types.EntityType(entitytype.Application.String()),
					ID:   "TinyTodo",
				},
			},
			wantErr: assert.NoError,
		},
		{
			name:    "invalid ID",
			euid:    "::",
			want:    EntityUID{},
			wantErr: assert.Error,
		},
		{
			name:    "malformed ID",
			euid:    "Application:\"TinyTodo\"",
			want:    EntityUID{},
			wantErr: assert.Error,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := Parse(tt.euid)
			if !tt.wantErr(t, err, fmt.Sprintf("Parse(%v)", tt.euid)) {
				return
			}
			assert.Equalf(t, tt.want, got, "Parse(%v)", tt.euid)
		})
	}
}
