package entitystore

import "github.com/cedar-policy/cedar-go"

type Entity interface {
	AsCedarEntity() *cedar.Entity
}
