package main

import "github.com/spf13/cobra"

func newTagCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "tag <id>",
		Short: "增删条目的用户标签",
		Args:  cobra.ExactArgs(1),
		RunE:  notImplemented,
	}
	cmd.Flags().StringSlice("add", nil, "要加的标签，逗号分隔")
	cmd.Flags().StringSlice("rm", nil, "要去掉的标签，逗号分隔")
	return cmd
}
