package main

import "github.com/spf13/cobra"

func newSearchCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "search <query>",
		Short: "搜索资料库",
		Args:  cobra.MinimumNArgs(1),
		RunE:  notImplemented,
	}
	cmd.Flags().String("src", "", "只看某个平台，如 douyin")
	cmd.Flags().String("in", "", "只在某一层命中，如 title、asr")
	cmd.Flags().String("kind", "", "只看某种类型：article | post | image | video | file")
	cmd.Flags().Bool("json", false, "以 JSON 输出")
	return cmd
}
