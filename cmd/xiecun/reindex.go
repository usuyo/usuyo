package main

import "github.com/spf13/cobra"

func newReindexCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "reindex",
		Short: "从文件重建搜索索引",
		Args:  cobra.NoArgs,
		RunE:  notImplemented,
	}
}
