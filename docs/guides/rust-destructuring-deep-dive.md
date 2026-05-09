# Rust 解构完全指南 —— 从 Misaka-Tauri 项目实战出发

**读者**：已读过 `rust-learning-faq-modules-and-lib.md` 第 3.1 节，对 `Ok(manager)` / `Some(manager)` 有初步概念，但希望系统掌握 Rust 的整个解构体系。

**目标**：读完本文后，你能够：
- 理解解构在不同场景下的写法（`match` / `if let` / `while let` / `let` / 函数参数 / 闭包参数）
- 看懂本项目 `src-tauri/src/` 中所有涉及解构的代码
- 知道什么时候该用 `ref`、什么时候不必用

---

## 一、先统一概念：解构 = 形状匹配 + 名字绑定

### 1.1 不要理解为「拆箱子」

FAQ 里用「拆开包装」来类比解构，这在入门时很有效，但它有一个局限：解构的本质不是「拆」，而是 **对形状的定义 + 对每一部分的命名**。

```rust
// 这段代码来自 src-tauri/src/lib.rs:46-49
match sidecar::SidecarManager::start(
    agent_dir.to_str().unwrap_or("agent"),
    app_config.sidecar_port,
) {
    Ok(manager) => Some(manager),
    Err(e) => {
        tracing::warn!("Python Sidecar failed to start: {}", e);
        None
    }
}
```

把这个 `match` 读成三件事：

| 步骤 | 具体行为 |
|------|----------|
| **1. 值** | `start()` 返回一个 `Result<SidecarManager, String>` |
| **2. 形状匹配** | 该值有两种可能的形状：`Ok(???)` 或 `Err(???)` |
| **3. 名字绑定** | 如果形状对上 `Ok(???)`，括号里的东西就叫 `manager`；对上 `Err(???)` 就叫 `e` |

**`Ok(manager)` 不是函数调用。** 括号在这里是「形状的一部分」，不是「传参」。这是初学者最常见的误解。

### 1.2 解构发生的五个位置

在 Rust 里，解构可以出现在任何一个「能够引入新名字」的地方：

| 位置 | 语法 | 本项目示例 |
|------|------|-----------|
| `match` 分支 | `模式 => 表达式` | `lib.rs:46-49` |
| `if let` | `if let 模式 = 表达式 { ... }` | `sidecar.rs:71` |
| `while let` | `while let 模式 = 表达式 { ... }` | `streaming.rs:131` |
| `let` 语句 | `let 模式 = 表达式;` | `crypto.rs:68` |
| 函数/闭包参数 | `\|模式\| 表达式` | 无处不在的 `\|row\| row.get(0)` |

这五个位置的模式语法 **完全统一** —— `match` 里能写的模式，在 `if let` 和 `let` 里也能写。

---

## 二、枚举解构 —— Rust 里最常见的形态

Rust 用枚举表示「这个值可能是几种不同形态」，因此解构枚举是日常操作。

### 2.1 `Option<T>` 解构

`Option` 是 Rust 标准库定义的枚举：

```rust
enum Option<T> {
    None,
    Some(T),
}
```

来自 `src-tauri/src/commands/session.rs:74-87` 的真实代码：

```rust
fn validate_working_dir(dir: Option<&str>) -> Result<Option<String>, String> {
    match dir {
        None | Some("") => Ok(None),       // ①
        Some(path) => {                     // ②
            let p = Path::new(path);
            if !p.exists() || !p.is_dir() {
                return Err(format!(
                    "Working directory does not exist or is not a directory: {}",
                    path
                ));
            }
            Ok(Some(path.to_string()))
        }
    }
}
```

三个要点：

**① 竖线 `|` 表示「或」——多个模式共享同一分支**

`None | Some("")` 读作：「如果是 `None`，或者 `Some` 里面是空字符串 `""`」。

注意：`Some("")` 是「解构 + 内部值精确匹配」的组合——不仅要求是 `Some` 变体，还要求里面的值是 `""`。如果里面的值不是 `""`，就落到下一个分支。

**② `Some(path)` 解构出内部值并命名为 `path`**

这不是「把 `dir` 变成 `path`」，而是：「`dir` 是 `Some(某个值)`」→「把那个值取出来，在这条分支里叫 `path`」。

**`match` 是 Rust 里唯一能「穷举检查」的解构位置**——如果漏了某个变体，编译器会报错。`Option` 只有两个变体（`Some` 和 `None`），所以 `match` 至少需要覆盖这两种情况。

### 2.2 `Result<T, E>` 解构

`Result` 的定义：

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

来自 `src-tauri/src/streaming.rs:141-151` 的真实代码（流式处理中的错误分支）：

```rust
match chunk {
    Ok(delta) => {                          // 解构出 delta
        if let Err(e) = self.handle_delta(delta) {
            tracing::warn!(error = %e, "Failed to emit stream event, continuing");
        }
    }
    Err(e) => {                             // 解构出 e
        self.emit_error(&e.to_string())?;
        return Err(anyhow::anyhow!("Stream error: {e}"));
    }
}
```

这里出现了 **两层解构嵌套**：

- 外层 `match` 解构 `Result`：`Ok(delta)` / `Err(e)`
- 内层 `if let` 解构另一个 `Result`：`if let Err(e) = self.handle_delta(delta)`

`if let` 的作用是「只匹配我关心的那一种形状，不匹配就什么都不做」。写成拆解：

```rust
if let Err(e) = self.handle_delta(delta) {
    // 只有 handle_delta 返回 Err 时，才进入这里
    // 如果是 Ok，直接跳过
}
```

### 2.3 自定义枚举解构 —— `AgentHandle`

这是本项目最复杂的枚举解构案例。来自 `src-tauri/src/services/llm/traits.rs:138-221`：

```rust
pub enum AgentHandle {
    OpenAi(Agent<rig::providers::openai::completion::CompletionModel>),
    Anthropic(Agent<rig::providers::anthropic::completion::CompletionModel>),
    Gemini(Agent<rig::providers::gemini::completion::CompletionModel>),
}

impl AgentHandle {
    pub async fn prompt(&self, input: &str) -> Result<String> {
        match self {
            Self::OpenAi(a) => a.prompt(input).await.map_err(|e| anyhow::anyhow!("{e}")),
            Self::Anthropic(a) => a.prompt(input).await.map_err(|e| anyhow::anyhow!("{e}")),
            Self::Gemini(a) => a.prompt(input).await.map_err(|e| anyhow::anyhow!("{e}")),
        }
    }
}
```

三个分支中的 `a` 是各自独立的——**每个分支有自己的作用域，名字可以重用**。`Self::OpenAi(a)` 里的 `a` 只在 `=>` 右侧有效；到 `Self::Anthropic(a)` 时，又是一个全新的 `a`。

这是 Rust 里一种经典模式：**用枚举包装不同类型，通过 `match` 解构后统一调用同名方法**。相当于把「运行时多态」用「编译期枚举分发」来实现——编译器会检查你是否覆盖了所有变体。

### 2.4 嵌套枚举解构

来自 `src-tauri/src/services/llm/traits.rs:80-98`：

```rust
match item {
    MultiTurnStreamItem::StreamAssistantItem(content) => match content {
        StreamedAssistantContent::Text(text) => {
            if text.text.is_empty() {
                None
            } else {
                Some(StreamDelta::Text(text.text))
            }
        }
        StreamedAssistantContent::Reasoning(reasoning) => {
            // ...
        }
    },
    // ...
}
```

这里 `match` 里面又套了一个 `match`——因为外层枚举包装的值本身又是一个枚举。读法遵循「从外到内、逐层解构」：

`item` 是 `MultiTurnStreamItem` → 如果是 `StreamAssistantItem` 变体 → 把里面的值叫 `content` → `content` 又是 `StreamedAssistantContent` → 再解构一层。

如果你觉得嵌套 `match` 可读性差，可以用 `if let` 扁平化，但编译器不介意嵌套——它一样做穷举检查。

---

## 三、`if let` ——「只关心一种形状」时的简写

`if let` 是 `match` 的语法糖。以下两个写法等价：

```rust
// 写法 A：match 完整分支
match Some(42) {
    Some(x) => println!("{}", x),
    None => {}  // 空分支
}

// 写法 B：if let 简写
if let Some(x) = Some(42) {
    println!("{}", x);
}
```

项目中几乎每个文件都有 `if let`。来看看几种典型形态：

### 3.1 基础用法：解构 `Option`

```rust
// src-tauri/src/crypto.rs:24
if let Ok(dir) = crate::config::config_dir() {
    parts.extend_from_slice(dir.to_string_lossy().as_bytes());
}
```

这里解构的是 `Result` 而非 `Option`，原理完全相同：成功时拿到 `dir`，失败时什么都不做。

### 3.2 解构 `Result` 的 `Err` 侧

```rust
// src-tauri/src/commands/session.rs:97
if let Err(e) = WorkspaceRepo::record_usage(conn, &id, dir, display_name.as_deref()) {
    tracing::warn!("Failed to record directory usage: {}", e);
}
```

`if let` 不限于匹配「成功」那一侧——你想匹配什么变体就写什么变体。这里是「只关心失败的情况」。

### 3.3 嵌套解构：直接从 `Some` 里拆出元组

```rust
// src-tauri/src/commands/chat.rs:243
if let Some((config_id, model_id)) = raw.split_once(':') {
    Ok(ModelSpec {
        config_id: config_id.to_string(),
        // ...
    })
}
```

`raw.split_once(':')` 返回 `Option<(&str, &str)>`——「可能有一个两元素的元组」。这条 `if let` 一步到位完成了 **两层解构**：
1. `Some(...)` → 从 `Option` 里取出内部值
2. `(config_id, model_id)` → 把内部的元组拆成两个名字

这就是解构「按形状匹配」的真正威力——你可以把形状描述得非常精确，编译器帮你把值同时绑定到多个名字。

---

## 四、`while let` ——「只要模式还匹配，就一直循环」

`while let` 把迭代和解构合二为一。来自 `src-tauri/src/services/llm/streaming.rs:131-151`：

```rust
while let Some(chunk) = stream.next().await {
    if self.abort_flag.load(Ordering::Relaxed) {
        tracing::info!("Stream aborted by user");
        break;
    }

    match chunk {
        Ok(delta) => {
            if let Err(e) = self.handle_delta(delta) {
                tracing::warn!(error = %e, "Failed to emit stream event, continuing");
            }
        }
        Err(e) => {
            self.emit_error(&e.to_string())?;
            return Err(anyhow::anyhow!("Stream error: {e}"));
        }
    }
}
```

逐句读：

- `stream.next().await` 返回 `Option<Result<StreamDelta, Error>>`
- `while let Some(chunk)` —— 只要 stream 还能产出下一个元素，就叫它 `chunk`；返回 `None` 时循环结束
- 进入循环体后，`chunk` 的类型是 `Result<StreamDelta, Error>`，再用 `match` 解构一层

如果你来自 JavaScript/TypeScript，可以类比 `for (const chunk of stream)`——但 Rust 的 `while let` 更通用，因为它可以对 **任何** 返回 `Option` 或 `Result` 的操作做循环，不限于迭代器。

---

## 五、元组解构 —— 同时赋予多个名字

元组是一组可能类型不同的值打包在一起。解构一个元组，就是一次性给每个位置起名。

### 5.1 `let` 解构元组

```rust
// src-tauri/src/crypto.rs:68
let (nonce_bytes, ciphertext) = combined.split_at(NONCE_LEN);
```

`split_at` 返回 `(&[u8], &[u8])`——一个两元素元组。左边 `(nonce_bytes, ciphertext)` 直接把两个分量各自命名为 `nonce_bytes` 和 `ciphertext`。不需要 `[0]` 和 `[1]`，也不需要有中间变量。

### 5.2 `if let` 解构元组

```rust
// src-tauri/src/db/repository/session_repo.rs:75
if let (Some(input), Some(output)) = (input_tokens, output_tokens) {
    conn.execute(
        "UPDATE sessions
         SET total_input_tokens = total_input_tokens + ?1,
             total_output_tokens = total_output_tokens + ?2,
             last_message_at = CURRENT_TIMESTAMP,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = ?3",
        rusqlite::params![input, output, session_id],
    )?;
}
```

这是进阶写法：**同时检查两个 `Option`**。

`(input_tokens, output_tokens)` 是一个元组 `(Option<u64>, Option<u64>)`。

- 用 `(Some(input), Some(output))` 去匹配——只有当 **两个都是 `Some`** 时才进入分支
- `input` 和 `output` 各自绑定到 `Some` 内部的值
- 如果任意一个是 `None`，整条 `if let` 不匹配，跳过

这比分别写两个 `if let` 再 `&&` 更安全——编译器保证你不会在某一侧忘记检查。

### 5.3 迭代中解构元组

```rust
// src-tauri/src/db/repository/settings_repo.rs:40
let (k, v) = row?;
```

`row?` 返回的是一个元组（比如 `(String, String)`），直接用模式拆成 `k` 和 `v`。

### 5.4 闭包参数中的元组解构

来自本项目 Rust 代码中常见模式：

```rust
// 效果等价：闭包拿到一个元组，直接拆成 (k, v)
map.iter().map(|(k, v)| format!("{}: {}", k, v))
```

`|(k, v)|` 直接在参数位置解构，省去 `|item| let (k, v) = item;` 这一层。

---

## 六、结构体解构 —— 按字段名取出值

结构体解构在项目中常见于类型转换（比如把一个 `StreamUsage` 拆成字段再重新组装）。

```rust
// 语法示例（结构体解构的标准形式）
let StreamUsage { input_tokens, output_tokens } = usage;
// 此后 input_tokens 和 output_tokens 直接可用
```

更常见的是 **同时重命名**：

```rust
// src-tauri/src/commands/chat.rs:358（简化示意）
match &result.usage {
    Some(usage) => {
        let StreamUsage { input_tokens, output_tokens } = usage;
        (Some(input_tokens), Some(output_tokens))
    }
    None => (None, None),
}
```

### 6.1 字段重命名

```rust
let StreamUsage { input_tokens: in_tok, output_tokens: out_tok } = usage;
//                ^^^^^^^^^^^^^^^^^^^   ^^^^^^^^^^^^^^^^^^^^^
//                字段名: 你想用的名字     字段名: 你想用的名字
```

当你想用不同于结构体字段名的变量名时，用 `字段: 新名字` 语法。

### 6.2 忽略不需要的字段：`..`

```rust
let Message { content, role, .. } = msg;
// content 和 role 被取出，其余字段全部忽略
```

如果漏写了 `..`，编译器会报错「你没有解构所有字段」——这是 Rust 的结构体解构必须穷举（或显式忽略）所有字段的规则。

---

## 七、引用与解构 —— `ref` / `ref mut` / `&` 的协同

这是新手最容易晕的部分。先记一条铁律：

> **模式左侧的 `ref` 表示「把匹配到的值借过来用，不要拿走所有权」。**

### 7.1 为什么要 `ref`？

考虑这段代码：

```rust
// src-tauri/src/sidecar.rs:71
impl Drop for SidecarManager {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.child {
            tracing::info!("Shutting down Python Sidecar...");
            let _ = child.kill();
        }
    }
}
```

注意写的是 `ref mut child`，不是 `child`。为什么？

- `self.child` 的类型是 `Option<std::process::Child>`
- 如果写 `Some(child)`，解构会 **把 `Child` 的所有权从 `self.child` 里移出来**——这在 `&mut self` 方法里是不允许的
- `ref mut child` 的意思是：「如果 `self.child` 是 `Some`，请让我能 **可变借用** 里面的值，暂时叫它 `child`，但不要拿走所有权」

`ref mut` 是两个修饰的组合：

| 写法 | 含义 |
|------|------|
| `Some(child)` | 把内部值 **移出**，所有权转移给 `child` |
| `Some(ref child)` | **不可变借用** 内部值，`child` 是 `&Child` 类型 |
| `Some(ref mut child)` | **可变借用** 内部值，`child` 是 `&mut Child` 类型 |

### 7.2 何时可以不加 `ref`？

当被解构的类型实现了 `Copy`（像 `i32`、`bool` 这些简单类型），移出也无所谓——编译器会自动复制一份。

```rust
if let Some(flag) = self.active_streams.get(session_id) {
    // flag 是 bool（实现了 Copy），不需要 ref
    flag.store(true, Ordering::Relaxed);
}
```

此外，如果模式前面有 `&`（对引用做解构），也不需要在里面写 `ref`：

```rust
// 这两个写法等价
match &x { Some(v) => ... }    // 匹配引用，自动得到 &T
match x { Some(ref v) => ... }  // 匹配值，手动 borrow
```

**本项目的实际选择**：大多数场景直接用 `if let Some(ref xxx)` 而不纠结，因为一眼看去意图明确。

### 7.3 项目中 `ref` 实际出现的场景

```rust
// src-tauri/src/commands/workspace.rs:19
if let Some(ref dir) = start_path {
    let p = Path::new(dir);
    if p.is_dir() {
        // ...
    }
}

// src-tauri/src/commands/chat.rs:214
let messages = if let Some(ref bid) = before_id {
    MessageRepo::find_before(&db, &session_id, bid, limit)
} else {
    // ...
};
```

这些地方的共性：**不想拿走 `Option` 内部值的所有权，只想借用一下**。因为后续可能还要用到 `start_path` 或 `before_id`。

---

## 八、`@` 绑定 —— 同时匹配和命名

`@` 运算符让你在解构时：一方面用子模式约束值，另一方面把整体绑定到一个名字。

```rust
match user_input {
    cmd @ ("quit" | "exit" | "q") => {
        println!("Received exit command: {}", cmd);
    }
    other => {
        println!("Processing: {}", other);
    }
}
```

`cmd @ ("quit" | "exit" | "q")` 做两件事：
1. 检查值是不是 `"quit"` / `"exit"` / `"q"` 之一
2. 如果是，把完整的值（比如 `"quit"`）绑定到 `cmd`

本项目代码中出现较少，但它是理解完整模式语法的必要环节。

---

## 九、闭包参数中的解构 —— 一行写完

项目中大量存在这种模式：

```rust
// filter_map 闭包中直接解构
.filter_map(|item| async move {
    match item {
        Ok(multi) => map_multi_turn_item(multi).map(Ok),
        Err(e) => Some(Err(anyhow::anyhow!("{e}"))),
    }
})
```

闭包的参数 `|item|` 本身就是一个模式位置。你可以直接在参数里做更深层的解构：

```rust
// 这行来自 message_repo.rs 的闭包（简化）
|row| Ok((row.get(0)?, row.get(1)?))
// row 是一个模式变量名（最简形式）

// 如果你愿意，也可以这样写（把迭代器元素的元组直接在参数里拆开）
collection.iter().map(|(key, value)| format!("{key}: {value}"))
```

---

## 十、匹配守卫（match guard）—— 在模式后面再加条件

有时你需要的不仅仅是形状匹配，还需要检查内部的值的属性：

```rust
match delta {
    StreamDelta::Text(text) if text.is_empty() => {
        // 文本为空，跳过
    }
    StreamDelta::Text(text) => {
        // 文本非空，正常处理
        self.emit_text_delta(&text)?;
    }
}
```

`模式 if 条件` 这种写法叫 **匹配守卫**。只有当模式匹配 **且** 后面的条件为 `true` 时，这条分支才会被选中。

本项目中的实际例子（`src-tauri/src/services/llm/traits.rs:80-87`）：

```rust
match content {
    StreamedAssistantContent::Text(text) => {
        if text.text.is_empty() {   // ← 用 if 在分支内部检查，而非 match guard
            None
        } else {
            Some(StreamDelta::Text(text.text))
        }
    }
}
```

这里用 `if` 而非 match guard，效果相同。match guard 更适合「你需要在匹配阶段就筛掉，否则会落入错误分支」的场景。

---

## 十一、解构的可驳性 —— 编译器如何帮你

Rust 把模式分为两类，这直接决定你可以在什么位置使用什么模式：

| 类型 | 特点 | 可用位置 |
|------|------|----------|
| **不可驳（irrefutable）** | 一定能匹配上 | `let`、函数参数、闭包参数、`for` 循环 |
| **可驳（refutable）** | 可能匹配不上 | `match`、`if let`、`while let` |

```rust
let (x, y) = (1, 2);            // 不可驳：元组一定能拆成两个元素
let Some(z) = maybe_value;      // ❌ 编译错误！let 要求不可驳模式
if let Some(z) = maybe_value {} // ✅ if let 接受可驳模式
```

这个限制背后的设计哲学是：**如果你写 `let`，你声明了「这个绑定一定成立」；编译器不让你在 `let` 里写可能失败的模式，是防止你在后续代码里遇到一个可能从未被初始化的变量。**

---

## 十二、本项目解构用法全景速查表

按你在阅读本项目代码时可能遇到的模式排序：

| 代码模式 | 出现位置 | 属于哪种解构 |
|----------|----------|-------------|
| `Ok(manager) => Some(manager)` | `lib.rs:50` | `match` + 枚举解构 |
| `Err(e) => { ... None }` | `lib.rs:51` | `match` + 枚举解构 |
| `None \| Some("") => Ok(None)` | `session.rs:76` | `match` + 枚举解构 + 或模式 + 字面量匹配 |
| `Some(path) => { ... }` | `session.rs:77` | `match` + 枚举解构 |
| `Self::OpenAi(a) => a.prompt(...)` | `traits.rs:149` | `match` + 枚举解构（变体带数据） |
| `if let Some(ref mut child) = self.child` | `sidecar.rs:71` | `if let` + `ref mut` |
| `if let Some(ref dir) = start_path` | `workspace.rs:19` | `if let` + `ref` |
| `if let Err(e) = some_function()` | `session.rs:97` | `if let` + Result 解构 |
| `if let Some((config_id, model_id)) = raw.split_once(':')` | `chat.rs:243` | `if let` + 嵌套解构（Option 包元组） |
| `if let (Some(in), Some(out)) = (a, b)` | `session_repo.rs:75` | `if let` + 元组解构 |
| `while let Some(chunk) = stream.next().await` | `streaming.rs:131` | `while let` + 枚举解构 |
| `let (nonce_bytes, ciphertext) = combined.split_at(NONCE_LEN)` | `crypto.rs:68` | `let` + 元组解构 |
| `match chunk { Ok(delta) => { ... } Err(e) => { ... } }` | `streaming.rs:141` | `match` + Result 解构 |
| `match delta { StreamDelta::Text(text) => ... }` | `streaming.rs:159` | `match` + 自定义枚举解构 |
| `match item { Ok(multi) => ... Err(e) => ... }` | `traits.rs:195` | 闭包内 `match` + Result 解构 |
| `match self { Self::OpenAi(a) => ... }` | `traits.rs:148` | `match` + 带泛型数据的枚举解构 |
| `match content { StreamedAssistantContent::Text(text) => ... }` | `traits.rs:81` | 嵌套 `match` + 枚举解构 |
| `\|row\| Ok((row.get(0)?, row.get(1)?))` | `message_repo.rs:150` | 闭包参数模式（始终不可驳） |
| `\|item\| async move { match item { Ok(multi) => ... } }` | `traits.rs:194` | 闭包参数 + 内部 match 解构 |

---

## 十三、学习路径建议

如果这是你第一次系统学 Rust 解构，建议按下列顺序动手：

1. **重读 `lib.rs:44-54` 的 Sidecar 启动代码**——用 `match` 解构 `Result` 的最基本形式。画流程图：`start()` → 返回 `Result` → `match` 分叉 → `Ok` 臂拿到 `manager` → 包成 `Some(manager)`。
2. **打开 `session.rs:74-87`**——观察 `None | Some("")` 和 `Some(path)` 两个分支，理解「或模式」和「内部值精确匹配」。
3. **打开 `traits.rs:138-152`**——看 `AgentHandle` 的三个 `match` 分支，理解「同一个名字 `a` 在不同分支里可以是不同类型」。
4. **打开 `streaming.rs:131-151`**——追踪 `while let` → `match` → `if let` 三层嵌套解构。
5. **回到你正在写的代码**——把 `match` 和 `if let` 用在你的 `Result` / `Option` 上。

---

## 十四、常见误区澄清

| 误区 | 事实 |
|------|------|
| 「`Ok(x)` 是在调用函数 `Ok`」 | `Ok(x)` 在模式位置是解构，在表达式位置才是构造。位置决定含义。 |
| 「模式里的名字必须先 `let` 声明」 | 模式里的名字直接在匹配成功时绑定，不需要提前声明。 |
| 「`Some` 是方法」 | `Some` 是枚举变体，`x.some()` 这种写法在 Rust 里不存在。 |
| 「`ref` 是多余的，用 `&` 就行了」 | `ref` 用于模式左侧，`&` 用于类型和表达式右侧。`let &x = &y;` 和 `let x = y;` 在简单场景下等价，但语义不同。 |
| 「解构就是把东西拆开」 | 更准确的理解是「定义形状 + 绑定名字」。形状不匹配时走下一个分支（或编译失败），匹配时才产生绑定。 |
| 「`if let` 只能解构 `Option`」 | 可以解构任何枚举，包括 `Result`、自定义枚举。 |
| 「函数参数不能解构」 | 函数参数和闭包参数都是模式位置，完全支持解构。 |

---

*本文基于 Misaka-Tauri (`D:\code\Misaka-Tauri\src-tauri\src`) 的最新代码编写，所有示例均来自项目实际代码。概念定义以 [The Rust Reference — Patterns](https://doc.rust-lang.org/reference/patterns.html) 和 [Rust By Example — Flow of Control](https://doc.rust-lang.org/rust-by-example/flow_control/match.html) 为准。*
