package main

import (
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server"
	"log/slog"
	"os"
)

func prepareLogger() server.Option {
	if logLevel := os.Getenv("GO_LOG"); logLevel != "" {
		var level slog.Level
		switch logLevel {
		case slog.LevelDebug.String():
			level = slog.LevelDebug
		case slog.LevelInfo.String():
			level = slog.LevelInfo
		case slog.LevelWarn.String():
			level = slog.LevelWarn
		case slog.LevelError.String():
			level = slog.LevelError
		}
		return server.WithLogger(
			slog.New(
				slog.NewTextHandler(
					os.Stdout,
					&slog.HandlerOptions{
						Level: level,
					},
				),
			),
		)
	}
	return server.WithLogger(DefaultLogger)
}
