package main

import "github.com/spf13/cobra"

func newJobsCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "jobs",
		Short: "查看后台任务",
		Args:  cobra.NoArgs,
		RunE:  notImplemented,
	}
	cmd.Flags().String("retry", "", "重试指定任务 id")
	return cmd
}
