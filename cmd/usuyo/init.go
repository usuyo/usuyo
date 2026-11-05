package main

import "github.com/spf13/cobra"

func newInitCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "init [dir]",
		Short: "新建资料库",
		Args:  cobra.MaximumNArgs(1),
		RunE:  notImplemented,
	}
}
