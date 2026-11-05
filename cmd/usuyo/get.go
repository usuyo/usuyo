package main

import "github.com/spf13/cobra"

func newGetCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "get <id>",
		Short: "查看一条内容",
		Args:  cobra.ExactArgs(1),
		RunE:  notImplemented,
	}
	cmd.Flags().String("layer", "all", "返回哪一层：original | derived | notes | all")
	return cmd
}
