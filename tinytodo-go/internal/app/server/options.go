package server

import (
	"log/slog"
)

type Option = func(s *Server)

func WithLogger(logger *slog.Logger) Option {
	return func(s *Server) {
		s.logger = logger
	}
}
