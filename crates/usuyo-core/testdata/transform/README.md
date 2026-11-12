# transform 样例

[003](../../../../docs/features/003-transforms.md#12-测试方案) 转换测试用的小文件，全是这台 Mac 上合成的（`say`、ffmpeg 的 `testsrc2` / `anullsrc`、AppKit 画的字），没有任何平台内容，也没有谁的录音。

| 文件 | 是什么 |
|---|---|
| `speech.m4a` | `say`（Reed，普通话）念「大家好，今天讲手冲咖啡的三段式注水。水温九十二度，粉水比一比十五。」，AAC 64k 单声道 16 kHz，约 7.43 秒；只有音频流 |
| `video-only.mp4` | `testsrc2` 320×180，h264，7.52 秒，没有音频流；和 `speech.m4a` 时长差 0.09 秒，符合配对规则 |
| `subtitle.zh.vtt` | 手写的两条 cue，当平台给的字幕 |
| `card.png` | 1080×1440，四行字：手冲咖啡三段式注水 / Pour over in 3 stages / 水温 92°C，粉水比 1:15 / 闷蒸 30 秒 Bloom |
| `card-copy.png` | `card.png` 的字节拷贝（测 `duplicate-images`） |
| `silent.mp4` | `card.png` 缩到 540×720 的静止画面，h264，5 秒，没有音频流 |
| `quiet.mp4` | `testsrc2` 加 `anullsrc` 的静音 AAC，5 秒 |

`fixture/` 是测试用的获取插件 `fixture/local`（清单、`plugin.sh`、每个样例的 fetch 输出）：`just dev-fixture-plugin` 把它和上面的文件拼成 `dash`（分开的画面、音轨和字幕）、`note`（两张一样的图）、`silent`、`quiet` 四条，装进 `USUYO_HOME` 的插件目录，`https://fixture.test/<名字>` 就取到它们。只给开发和测试用。

重新生成：`./make.sh`（只在 macOS 上；要 `say`、Swift、带 libx264 的 ffmpeg，默认 `/opt/homebrew/bin/ffmpeg`，可用 `FFMPEG=` 换）。`card.swift` 是画字的脚本。CI 直接用提交的文件，不重新生成；每次生成的字节不保证一样。
