### 使用方式
在当前可执行文件目录下 打开终端，输入以下命令：
```bash
mobius start
```
然后
| 操作系统 | 默认配置目录                         | 示例                                         |
|----------|----------|----------|
Linux   | `$XDG_CONFIG_HOME` or `$HOME`/.config | /home/alice/.config                      |
macOS   | `$HOME`/Library/Application Support   | /Users/Alice/Library/Application Support |
Windows | `{FOLDERID_LocalAppData}`             | C:\Users\Alice\AppData\Local             |

在上述系统对应下的`session_manager`目录下的config.json文件,添加进自己的deepseek api key;