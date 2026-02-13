# 红中 (Red Center) - Roguelike 麻将游戏 实施计划

## Context

创建一款灵感来自 Balatro(小丑牌) 的 Roguelike 麻将游戏。玩家在每关的 3 个小关(小盲注/大盲注/Boss)中,通过从待选区选牌、累积到出牌区凑出 14 张和牌来获得分数。凑出特殊牌型可获得更高倍率。美术风格为马赛克像素风 2D。

### 核心规则确认
- **待选区**: 默认 8 张牌,可通过道具增加
- **出牌方式**: 累积出牌,每次选几张从待选区移到出牌区,4次机会内凑满14张
- **操作次数**: 4次出牌 + 4次弃牌(独立计数)
- **弃牌**: 选中待选区的牌弃掉,从牌山补牌,消耗弃牌次数
- **得分**: 底注 × 倍率,凑出牌型获得更高倍率
- **技术**: Bevy 0.18, Rust, 2D pixel-art

---

## 架构概览

### 项目结构
```
red-center/
├── Cargo.toml
├── assets/
│   └── fonts/
│       └── pixel.ttf               # 像素风中文字体
├── src/
│   ├── main.rs                     # App 入口, 窗口配置, 插件注册
│   ├── plugins/
│   │   ├── mod.rs                  # RedCenterPluginGroup
│   │   ├── game.rs                 # AppState + PlayPhase 状态机
│   │   ├── tile.rs                 # 麻将牌生成、洗牌、位置更新
│   │   ├── board.rs                # 摸牌、出牌、弃牌、补牌逻辑
│   │   ├── input.rs                # 鼠标点击、牌选择
│   │   ├── scoring.rs              # 和牌判定、牌型识别、分数计算
│   │   └── ui.rs                   # UI 布局 + 更新
│   ├── components/
│   │   ├── mod.rs
│   │   ├── tile.rs                 # Tile, TileId, TileSuit, TileLocation
│   │   ├── board.rs                # UI 区域标记组件
│   │   └── game.rs                 # MainCamera 标记
│   ├── events.rs                   # 所有事件 (Event + Clone)
│   └── resources.rs                # GameState, TileWall, PlayerHand, PlayBoard
└── plans/
    └── structure.png
```

### 状态机
```
AppState::Menu → AppState::Playing → AppState::GameOver
                      │
                PlayPhase::Selecting   (玩家选牌/出牌/弃牌)
                PlayPhase::Scoring     (计算得分)
                PlayPhase::RoundResult (显示结果, 进入下一小关或游戏结束)
```

### 核心设计决策
1. **牌 = 世界空间 Sprite**(非 UI 节点): 便于动画、点击检测、视觉效果
2. **UI Chrome = Bevy Node 系统**: 边框、文字、按钮用 Bevy UI
3. **事件用 Observer 模式**: `commands.trigger()` + `add_observer()` (Bevy 0.17+)
4. **和牌算法 = [u8; 34] 计数数组 + 递归回溯**: 标准麻将 AI 方案

---

## Phase 1: 项目骨架与状态机

**目标**: 最小可编译 Bevy 0.18 应用,窗口显示,游戏状态切换

### 创建文件

**`Cargo.toml`**
```toml
[package]
name = "red-center"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = "0.18"
rand = "0.8"

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3
```

**`src/main.rs`** - 窗口 1280x720, `ImagePlugin::default_nearest()` 像素风, `Msaa::Off`

**`src/plugins/game.rs`** - 定义:
- `AppState` { Menu, Playing, GameOver } (States)
- `PlayPhase` { Selecting, Scoring, RoundResult } (SubStates of Playing)
- `setup_camera` 系统 (spawn Camera2d)

**`src/plugins/mod.rs`** - `RedCenterPluginGroup` 注册所有插件

**其他插件文件** - 空 Plugin 骨架

### 验证
- `cargo check` 编译通过
- `cargo run` 打开 1280x720 窗口, 标题 "红中 Red Center"

---

## Phase 2: 麻将牌数据模型与渲染

**目标**: 定义 136 张麻将牌,生成实体,用矩形+文字占位渲染

### 关键文件

**`src/components/tile.rs`** - 核心数据模型:
```rust
enum TileSuit { Man, Pin, Sou, Wind, Dragon }  // 万/筒/条/风/箭
struct TileId { suit: TileSuit, value: u8 }      // 牌面标识
struct Tile { id: TileId, copy_index: u8 }       // ECS Component (含副本编号)
enum TileLocation { Wall, Hand, Board, Discarded } // 牌的位置状态
struct TileSelected;                              // 选中标记
```

`TileId` 方法: `label()` 返回中文名("3万","中"), `suit_color()` 返回花色颜色, `is_simple()`/`is_terminal()`/`is_honor()` 分类判断

`Tile::generate_full_set()` → 生成 136 张牌 (万筒条各36 + 风16 + 箭12)

**`src/resources.rs`**:
```rust
struct TileWall { tiles: Vec<Entity> }   // 牌山
struct PlayerHand { tiles: Vec<Entity> }  // 待选区 (默认8张)
struct PlayBoard { tiles: Vec<Entity> }   // 出牌区 (最多14张)
struct GameState {
    level: u32, sub_round: SubRound,
    plays_remaining: u32,    // 出牌次数 (4)
    discards_remaining: u32, // 弃牌次数 (4)
    target_score: u32, current_score: u32,
    hand_size: usize,        // 待选区容量 (默认8, 可通过道具增加)
}
```

**`src/plugins/tile.rs`**:
- `spawn_tiles` 系统: 生成 136 个 Sprite 实体 (米色矩形 48x64px + Text2d 子实体显示文字)
- `update_tile_positions` 系统: 根据 TileLocation + 在 Vec 中的 index 计算 xy 位置
- 常量: `TILE_WIDTH=48`, `TILE_HEIGHT=64`, `TILE_GAP=4`

### 验证
- 进入 Playing 状态后 136 个牌实体被创建
- 手动将几张牌设为 `TileLocation::Hand` 可看到渲染

---

## Phase 3: UI 布局 (匹配原型图)

**目标**: 实现完整 UI 框架,匹配 `plans/structure.png`

### UI 层级
```
根节点 (全屏, Column)
├── 小丑区 (顶部横条, 高80px, 边框) → "小丑区 (放置道具, 目前先空白)"
├── 中间行 (Row, flex_grow: 1)
│   ├── 得分区 (左, 200px宽)
│   │   ├── "底注: {base}"
│   │   ├── "× 倍率: {mult}"
│   │   ├── "= {total}"
│   │   ├── "目标: {target}"
│   │   ├── "出牌: {plays}/4  弃牌: {discards}/4"
│   │   └── "{小盲注/大盲注/Boss}"
│   ├── 出牌区 (中, flex_grow: 1) → 14个空槽位边框
│   └── 牌山 (右, 120px宽) → "{count} 张"
├── 待选区 (底部中间, 带边框) → 牌的 Sprite 在此区域渲染
└── 按钮行 (Row, 居中)
    ├── 菜单
    ├── 出牌
    └── 弃牌
```

**`src/components/board.rs`** - UI 标记组件:
`JokerArea`, `ScoreArea`, `PlayArea`, `SelectionArea`, `TileWallDisplay`,
`MenuButton`, `PlayButton`, `DiscardButton`,
`BaseScoreText`, `MultiplierText`, `TotalScoreText`, `WallCountText`, `PlaysRemainingText`, `TargetScoreText`, `SubRoundText`

**`src/plugins/ui.rs`**:
- `setup_menu_ui`: 菜单页 - 标题 "红中" + "开始游戏" 按钮
- `setup_game_ui`: 游戏页全部 UI 布局 (OnEnter Playing)
- `update_score_display`: 监听 GameState 变化更新文字
- `update_wall_count`: 更新牌山剩余数
- `button_interaction_system`: 按钮悬停/点击视觉反馈 + 触发事件

### 验证
- 菜单页显示标题和开始按钮
- 点击开始进入游戏,所有 UI 区域按原型图位置显示
- 按钮有悬停/点击颜色反馈

---

## Phase 4: 核心游戏循环

**目标**: 完整的 选牌→出牌/弃牌→补牌 循环

### 游戏流程
```
进入 Playing → 初始化(洗牌, 摸8张到待选区) → PlayPhase::Selecting

Selecting 阶段:
  点击待选区的牌 → 切换选中状态(牌上移15px高亮)
  点击 "出牌" → 选中牌移到出牌区, 从牌山补牌, plays_remaining--
  点击 "弃牌" → 选中牌弃掉, 从牌山补牌, discards_remaining--

  出牌区达到14张 OR plays_remaining==0 → PlayPhase::Scoring
```

### 关键文件

**`src/events.rs`**:
```rust
#[derive(Event, Clone)] struct TileSelectedEvent { tile_entity: Entity }
#[derive(Event, Clone)] struct TilePlayedEvent;
#[derive(Event, Clone)] struct TileDiscardedEvent;
#[derive(Event, Clone)] struct ScoreCalculatedEvent { base: u32, mult: u32, total: u32 }
#[derive(Event, Clone)] struct RoundEndedEvent { passed: bool }
```

**`src/plugins/input.rs`**:
- `handle_tile_click`: 鼠标点击 → 世界坐标转换 → AABB 碰撞检测 → 触发 TileSelectedEvent
- `on_tile_selected` (observer): 切换 TileSelected 组件

**`src/plugins/board.rs`**:
- `draw_initial_hand`: OnEnter(Playing), 从牌山摸 hand_size 张到待选区
- `on_play_tiles` (observer): 选中牌 Hand→Board, 从牌山补牌至 hand_size, plays_remaining--
- `on_discard_tiles` (observer): 选中牌 Hand→Discarded, 从牌山补牌至 hand_size, discards_remaining--
- `update_tile_visual_selection`: 选中的牌上移显示, 取消选中复位
- `check_phase_transition`: 出牌区14张 or plays==0 → 进入 Scoring

### 验证
- 开局 8 张牌显示在待选区
- 点击牌切换选中状态(上移高亮)
- 出牌: 选中牌移到出牌区, 新牌补充到待选区
- 弃牌: 选中牌消失, 新牌补充
- 出牌/弃牌次数正确减少
- 14张满或出牌次数耗尽自动进入计分

---

## Phase 5: 和牌判定与计分算法

**目标**: 判断出牌区 14 张牌是否构成和牌,识别牌型,计算得分

### 算法设计

**计数数组**: `[u8; 34]` - 索引映射:
- 0-8: 万1-9, 9-17: 筒1-9, 18-26: 条1-9
- 27-30: 东南西北, 31-33: 中发白

**和牌判定 (递归回溯)**:
1. 先检查特殊牌型: 国士无双(13张幺九+1对), 七对子(7对)
2. 标准和牌: 遍历 34 种可能的雀头(对子) → 剩余 12 张用递归提取 4 个面子(顺子/刻子)
3. 从最小索引开始,优先尝试刻子,再尝试顺子

**牌型识别与倍率**:
| 牌型 | 条件 | 倍率 |
|------|------|------|
| 平和 | 基本和牌(无特殊牌型) | ×1 |
| 断幺九 | 全部是 2-8 的数牌 | ×2 |
| 一气通贯 | 同花色 123+456+789 | ×3 |
| 对对和 | 4组刻子+1对 | ×4 |
| 混一色 | 单一数牌花色+字牌 | ×5 |
| 清一色 | 纯单一数牌花色 | ×8 |
| 七对子 | 7个对子 | ×4 |
| 国士无双 | 13种幺九各1+任一幺九对 | ×13 |

**得分计算**: `score = base_ante × best_multiplier`
- 不到 14 张或非和牌: 只得基础分 (base × 1)
- 和牌: base × 最高牌型倍率
- 多个牌型取最高倍率

**`src/plugins/scoring.rs`**:
- `evaluate_hand(tiles: &[TileId]) -> HandResult` - 纯函数,返回是否和牌+牌型列表+最佳倍率
- `calculate_score` 系统 (OnEnter Scoring): 读取出牌区牌面 → evaluate → 更新 GameState
- 包含 `#[cfg(test)] mod tests` 单元测试

### 验证
- 单元测试覆盖: 平和/七对子/国士无双/对对和/清一色/混一色/一气通贯
- 非和牌14张返回 is_winning: false
- 不到14张返回 is_winning: false
- 游戏中实际计分正确显示

---

## Phase 6: Roguelike 关卡进度

**目标**: 盲注系统、关卡推进、游戏结束与重新开始

### 进度规则
```
Level N:
  小盲注 目标: 100 × N
  大盲注 目标: 200 × N
  Boss   目标: 400 × N

同一关内: 每小关清空出牌区, 牌山保持 (战略资源管理)
过关后: 重置整副牌 (新的136张)
失败: 游戏结束
```

### 实现内容

**`src/plugins/game.rs`** 扩展:
- `advance_sub_round()`: SmallBlind → BigBlind → Boss → 过关
- `advance_level()`: level++, 重置为 SmallBlind
- `reset_for_sub_round()`: 重置出牌/弃牌次数, 清空出牌区, 更新目标分

**`src/plugins/ui.rs`** 扩展:
- 小关开始横幅: "小盲注 - 目标: 100" (1.5秒后消失)
- 游戏结束页面: "游戏结束" + 最终分数 + "重新开始" 按钮

**新关卡初始化**:
- 销毁所有旧牌实体
- 重新生成洗牌的 136 张
- 摸初始手牌

### 验证
- 通过小盲注 → 自动进入大盲注(目标翻倍)
- 通过 Boss → 进入 Level 2(分数翻倍)
- 失败 → 游戏结束页面
- 重新开始 → 回到菜单
- 小关切换时出牌区清空,牌山保留
- 过关时全部重置

---

## 开发前准备

### 环境检查
```bash
rustup default stable    # 确保 Rust 工具链配置
rustc --version          # 验证 Rust 可用
```

### 字体资源
需要下载一个支持中文的像素风字体放到 `assets/fonts/pixel.ttf`。
推荐:
- Zpix (最像素): https://github.com/SolidZORO/zpix-pixel-font
- 文泉驿点阵宋体

---

## 总验证计划

每个 Phase 完成后:
1. `cargo check` 编译通过
2. `cargo run` 功能可视化验证
3. Phase 5 增加单元测试: `cargo test`

最终验证:
- 从菜单开始 → 进入游戏 → 选牌出牌 → 凑和牌得分 → 通过3小关 → 进入下一关
- 失败路径: 分数不够 → 游戏结束 → 重新开始
