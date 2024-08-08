package main

import (
	"log/slog"
	"os"
)

func main() {
	if err := startServer(); err != nil {
		DefaultLogger.Error("failed to start server", slog.Any("error", err))
		os.Exit(-1)
	}
}
