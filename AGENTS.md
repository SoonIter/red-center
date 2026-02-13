# 红中 Red Center

Roguelike 麻将游戏，灵感来自 Balatro（小丑牌）。Bevy 0.18 + Rust，2D 像素风。

## 技术栈

- **引擎**: Bevy 0.18（注意：不是 0.17，API 有较大差异）
- **语言**: Rust (edition 2021)
- **依赖**: bevy 0.18, rand 0.8
- **字体**: assets/fonts/pixel.ttf (Zpix 像素中文字体)
- **窗口**: 1280×720, 不可拉伸

## 构建命令

```bash
cargo check          # 快速编译检查
cargo run            # 运行游戏
cargo test           # 运行单元测试（scoring 模块）
```

## 项目结构

```
src/
├── main.rs                 # App 入口，窗口配置，插件注册
├── events.rs               # 所有游戏事件（Event + Clone）
├── resources.rs            # GameState, TileWall, PlayerHand, PlayBoard
├── components/
│   ├── tile.rs             # Tile, TileId, TileSuit, TileLocation, TileSelected
│   ├── board.rs            # UI 区域标记组件（ScoreArea, PlayButton 等）
│   └── game.rs             # MainCamera 标记
└── plugins/
    ├── mod.rs              # RedCenterPluginGroup
    ├── game.rs             # AppState + PlayPhase 状态机
    ├── tile.rs             # 牌生成、洗牌、位置更新
    ├── board.rs            # 摸牌、出牌、弃牌、补牌逻辑
    ├── input.rs            # 鼠标点击、牌选择
    ├── scoring.rs          # 和牌判定、牌型识别、分数计算
    └── ui.rs               # UI 布局 + 更新（菜单/游戏/结束三屏）
```

## 架构要点

### 状态机

```
AppState::Menu → AppState::Playing → AppState::GameOver
                      │
                PlayPhase::Selecting   (选牌/出牌/弃牌)
                PlayPhase::Scoring     (计算得分)
                PlayPhase::RoundResult (显示结果)
```

### 核心设计

- **牌 = 世界空间 Sprite**（Sprite::from_color + Text2d 子实体），不是 UI 节点
- **UI = Bevy Node 系统**（Text, Button, Node, BackgroundColor 等）
- **事件 = Observer 模式**：`commands.trigger()` 触发，`add_observer()` 注册处理
- **和牌算法 = [u8; 34] 计数数组 + 递归回溯**

### 游戏规则

- 待选区默认 8 张牌（hand_size，可通过道具增加）
- 4 次出牌机会 + 4 次弃牌机会（独立计数）
- 累积出牌到出牌区，凑满 14 张判定和牌
- 得分 = 底注 × 倍率（牌型决定倍率）
- 每关 3 小关：小盲注(100×N) / 大盲注(200×N) / Boss(400×N)

## Bevy 0.18 注意事项

这些与 0.17 的 skill/文档有差异，编码时务必注意：

1. **Msaa** 是 Component（挂在 Camera 上），不是 Resource
2. **WindowResolution** 用 `(u32, u32)` 不是 `(f64, f64)`
3. **with_children() 回调类型** 是 `ChildSpawnerCommands`（不是 `ChildSpawner`），不在 prelude 中，需要单独导入
4. **Event 必须 derive Clone**（Observer 模式要求）
5. **Sprite::from_color(color, Vec2)** 创建纯色矩形
6. **UI 文字用 Text::new()**, 世界文字用 **Text2d::new()**

## 实施进度

详见 `plans` 文件夹。
