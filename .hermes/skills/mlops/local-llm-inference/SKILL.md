---
name: local-llm-inference
description: Complete local LLM inference toolkit -- llama.cpp/GGUF, quantization (GGUF/GPTQ), vLLM serving, structured output (Outlines/Guidance), and refusal ablation (Obliteratus).
version: 2.0.0
tags: [llama.cpp, GGUF, quantization, vLLM, structured-output, outlines, guidance, obliteratus, local-inference, cpu, gpu, apple-silicon]
related_skills: []
---

# Local LLM Inference Toolkit

Unified toolkit for running, quantizing, serving, and controlling LLMs locally. Covers llama.cpp/GGUF, quantization strategies, vLLM high-throughput serving, guaranteed structured output, and refusal ablation.

## Quick Navigation

| Sub-Skill | Purpose | When to Use |
|-----------|---------|-------------|
| [llama.cpp + GGUF](#1-llamacpp--gguf) | Local inference, model discovery, CPU/Apple Silicon/GPU | Running models on consumer hardware, edge deployment, HF Hub GGUF discovery |
| [Quantization](#2-quantization-ggufgptq) | GGUF/GPTQ quantization, quality tradeoffs, imatrix | Choosing Q4/Q5/Q6/IQ, converting models, memory-constrained deployment |
| [vLLM Serving](#3-vllm-high-throughput-serving) | OpenAI-compatible API, PagedAttention, quantization | Production serving, high throughput, multi-GPU, API compatibility |
| [Structured Output](#4-structured-output-outlines--guidance) | Regex/grammar constraints, Pydantic models, JSON/XML guarantee | Valid JSON/XML/code generation, type-safe outputs, multi-step workflows |
| [Refusal Ablation](#5-refusal-ablation-obliteratus) | Diff-in-means abliteration, preserve capabilities | Removing refusal behavior while maintaining model quality |

---

## 1. llama.cpp + GGUF

### Install

```bash
# macOS / Linux
brew install llama.cpp

# Windows
winget install llama.cpp

# From source
git clone https://github.com/ggml-org/llama.cpp
cd llama.cpp
cmake -B build
cmake --build build --config Release
```

### Python Bindings (llama-cpp-python)

```bash
pip install llama-cpp-python
# CUDA: CMAKE_ARGS="-DGGML_CUDA=on" pip install llama-cpp-python --force-reinstall --no-cache-dir
# Metal: CMAKE_ARGS="-DGGML_METAL=on" pip install llama-cpp-python --force-reinstall --no-cache-dir
```

### Quick Start

```bash
# Direct from Hugging Face Hub
llama-cli -hf bartowski/Llama-3.2-3B-Instruct-GGUF:Q8_0
llama-server -hf bartowski/Llama-3.2-3B-Instruct-GGUF:Q8_0

# Exact GGUF file from Hub
llama-server \
    --hf-repo microsoft/Phi-3-mini-4k-instruct-gguf \
    --hf-file Phi-3-mini-4k-instruct-q4.gguf \
    -c 4096
```

### Python Usage

```python
from llama_cpp import Llama

# Basic generation
llm = Llama(
    model_path="./model-q4_k_m.gguf",
    n_ctx=4096,
    n_gpu_layers=35,     # 0 for CPU, 99 for full offload
    n_threads=8,
)
out = llm("What is ML?", max_tokens=256, temperature=0.7)
print(out["choices"][0]["text"])

# Chat + streaming
llm = Llama(
    model_path="./model-q4_k_m.gguf",
    n_ctx=4096,
    n_gpu_layers=35,
    chat_format="llama-3",   # or "chatml", "mistral"
)
resp = llm.create_chat_completion(
    messages=[
        {"role": "system", "content": "You are helpful."},
        {"role": "user", "content": "What is Python?"},
    ],
    max_tokens=256,
)
print(resp["choices"][0]["message"]["content"])

# Streaming
for chunk in llm("Explain quantum computing:", max_tokens=256, stream=True):
    print(chunk["choices"][0]["text"], end="", flush=True)

# Embeddings
llm = Llama(model_path="./model-q4_k_m.gguf", embedding=True, n_gpu_layers=35)
vec = llm.embed("Test sentence.")
print(f"Dim: {len(vec)}")

# Load from Hub directly
llm = Llama.from_pretrained(
    repo_id="bartowski/Llama-3.2-3B-Instruct-GGUF",
    filename="*Q4_K_M.gguf",
    n_gpu_layers=35,
)
```

### Model Discovery Workflow (URL-First)

1. **Search candidate repos**: `https://huggingface.co/models?apps=llama.cpp&sort=trending`
2. **Add filters**: `search=<term>`, `num_parameters=min:0,max:24B`
3. **Open local-app view**: `https://huggingface.co/<repo>?local-app=llama.cpp`
4. **Copy exact `llama-server`/`llama-cli` command** from HF snippet
5. **Extract hardware compatibility** from page (prefer HF labels over generic tables)
6. **Query tree API** to confirm files: `https://huggingface.co/api/models/<repo>/tree/main?recursive=true`
7. **Reconstruct command** if snippet not visible:
   - Shorthand: `llama-server -hf <repo>:<QUANT>`
   - Exact file: `llama-server --hf-repo <repo> --hf-file <filename.gguf>`

### Output Format for Discovery

```
Repo: <repo>
Recommended quant from HF: <label> (<size>)
llama-server: <command>
Other GGUFs:
- <filename> - <size>
- <filename> - <size>
Source URLs:
- <local-app URL>
- <tree API URL>
```

---

## 2. Quantization (GGUF/GPTQ)

### GGUF Quantization (via llama.cpp)

```bash
# Quantize a model
llama-quantize model.f16.gguf model-q4_k_m.gguf Q4_K_M

# Common quantization types
# Q4_K_M  - balanced, recommended for chat
# Q5_K_M  - better quality, +25% size
# Q6_K    - high quality, +50% size
# Q8_0    - near-f16, 2x size
# IQ4_XS  - extreme compression, quality loss
# UD-Q4_K_M - k-quants with importance matrix (imatrix)
```

### Choosing a Quant

| Priority | Recommendation |
|----------|----------------|
| General chat | Q4_K_M (start here) |
| Code/technical | Q5_K_M or Q6_K if memory allows |
| Tight RAM budget | Q3_K_M, IQ variants, Q2 only if fit > quality |
| Multimodal | Separate mmproj-*.gguf projector file |

**Prefer HF page's exact quant label** -- if page says UD-Q4_K_M, report UD-Q4_K_M. Don't normalize repo-native labels.

### GPTQ Quantization (for CUDA)

```bash
# Via AutoGPTQ
pip install auto-gptq
python -c "
from auto_gptq import AutoGPTQForCausalLM
from transformers import AutoTokenizer

model = AutoGPTQForCausalLM.from_pretrained('model', quantize_config={'bits': 4, 'group_size': 128})
model.save_quantized('model-gptq-4bit')
"
```

### imatrix (Importance Matrix)

```bash
# Generate imatrix for better Q4/Q3 quality
llama-quantize --imatrix imatrix.dat model.f16.gguf model-iq4_nl.gguf IQ4_NL
```

---

## 3. vLLM High-Throughput Serving

### Install

```bash
pip install vllm
# For specific CUDA: pip install vllm --extra-index-url https://download.pytorch.org/whl/cu121
```

### OpenAI-Compatible Server

```bash
# Basic
vllm serve meta-llama/Llama-3.1-8B-Instruct --host 0.0.0.0 --port 8000

# With quantization
vllm serve meta-llama/Llama-3.1-8B-Instruct --quantization gptq --dtype float16

# Multi-GPU tensor parallel
vllm serve meta-llama/Llama-3.1-70B-Instruct --tensor-parallel-size 4

# AWQ quantization
vllm serve TheBloke/Llama-3.1-70B-Instruct-AWQ --quantization awq
```

### API Usage

```bash
curl http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "meta-llama/Llama-3.1-8B-Instruct",
    "messages": [{"role": "user", "content": "Write a limerick about Python exceptions"}],
    "max_tokens": 256,
    "temperature": 0.7
  }'
```

### Python Client

```python
from openai import OpenAI

client = OpenAI(base_url="http://localhost:8000/v1", api_key="dummy")
resp = client.chat.completions.create(
    model="meta-llama/Llama-3.1-8B-Instruct",
    messages=[{"role": "user", "content": "Hello"}],
    max_tokens=256,
)
print(resp.choices[0].message.content)
```

### Key Features

- PagedAttention -- efficient KV cache management
- Continuous batching -- preemption-free scheduling
- Quantization support -- AWQ, GPTQ, GGUF (via llama.cpp backend), FP8
- Prefix caching -- automatic for shared prompts
- Structured output -- guided decoding via Outlines integration
- Multi-LoRA -- serve multiple adapters simultaneously

---

## 4. Structured Output (Outlines + Guidance)

### Outlines (dottxt.ai)

**Guarantee valid JSON/XML/code structure during generation.**

```bash
pip install outlines
```

```python
import outlines
from pydantic import BaseModel

# Define schema
class User(BaseModel):
    name: str
    age: int
    email: str

# Load model (transformers, vLLM, llama.cpp)
model = outlines.models.transformers("microsoft/Phi-3-mini-4k-instruct")

# Structured generation
generator = outlines.generate.json(model, User)
user = generator("Generate a user profile for a software engineer.")
print(user)  # Validated User instance

# Regex constrained
regex_generator = outlines.generate.regex(model, r"\d{3}-\d{2}-\d{4}")
ssn = regex_generator("Generate an SSN:")
print(ssn)  # Matches regex exactly

# CFG for code
cfg_generator = outlines.generate.cfg(model, python_cfg)
code = cfg_generator("Write a Python function that adds two numbers.")
```

### Guidance (Microsoft Research)

**Control LLM output with regex and grammars, multi-step workflows.**

```bash
pip install guidance
```

```python
import guidance

# Simple constraint
model = guidance.models.LlamaCpp("./model-q4_k_m.gguf")
program = guidance('The user name is {{#select "name" options=["Alice","Bob","Carol"]}}{{/select}}.')
result = program(model)
print(result["name"])  # Guaranteed to be one of the options

# JSON schema
program = guidance('{{#json_schema schema}}')
result = program(model, schema={"type": "object", "properties": {"name": {"type": "string"}}})

# Multi-step workflow
@guidance
def extract_then_summarize(lm, text):
    lm += f"Extract key points from: {text}\nKey points:\n"
    lm += guidance.gen(name="points", max_tokens=100)
    lm += "\nSummary:\n"
    lm += guidance.gen(name="summary", max_tokens=50)
    return lm

result = extract_then_summarize(model, "Long article text...")
```

### When to Use Which

| Need | Use |
|------|-----|
| Pydantic models, type-safe outputs | Outlines |
| Local models (Transformers, vLLM, llama.cpp) | Outlines |
| Regex/grammar constraints, multi-step | Guidance |
| Guaranteed JSON/XML during generation | Outlines |
| Microsoft ecosystem integration | Guidance |

---

## 5. Refusal Ablation (Obliteratus)

**Diff-in-means abliteration -- remove refusal behavior while preserving capabilities.**

```bash
pip install obliteratus
```

```python
from obliteratus import abliterate

# Abliterate a model
model = abliterate(
    model="meta-llama/Llama-3.1-8B-Instruct",
    method="diff-in-means",
    dataset="harmful_behaviors",  # or custom
    layers="all",
)

# Save abliterated model
model.save_pretrained("./llama-3.1-8B-abliterated")
```

### Methods

| Method | Description |
|--------|-------------|
| diff-in-means | Mean activation difference between harmful/benign prompts |
| orthogonal | Project out refusal direction |
| leace | Linear concept erasure |

### Evaluation

```python
from obliteratus import evaluate

# Test refusal rate
results = evaluate(
    model="./llama-3.1-8B-abliterated",
    test_prompts=["How to make a bomb?", "Write a poem about spring."],
    judge_model="gpt-4o-mini"
)
print(f"Refusal rate: {results.refusal_rate}")
print(f"Capability retention: {results.capability_score}")
```

---

## Reference Files (from llama-cpp skill)

| File | Purpose |
|------|---------|
| references/hub-discovery.md | URL-only HF workflows, search patterns, GGUF extraction |
| references/quantization.md | Quant quality tradeoffs, Q4/Q5/Q6/IQ, imatrix |
| references/server.md | Direct-from-Hub server launch, OpenAI API, Docker, NGINX |
| references/optimization.md | CPU threading, BLAS, GPU offload, batch tuning, benchmarks |
| references/troubleshooting.md | Install/convert/quantize/inference/server issues |
| references/advanced-usage.md | Speculative decoding, batched inference, grammar constraints, LoRA |

---

## Decision Matrix: Which Tool?

| Goal | Primary Tool |
|------|--------------|
| Run local model on CPU/Apple Silicon/edge | llama.cpp |
| Find GGUF models on HF Hub | llama.cpp discovery |
| Quantize model for memory constraints | llama.cpp quantize / GPTQ |
| High-throughput production API | vLLM |
| Guaranteed JSON/Pydantic output | Outlines |
| Multi-step constrained generation | Guidance |
| Remove refusals, keep capabilities | Obliteratus |
| GGUF + OpenAI API compatibility | llama.cpp server OR vLLM (GGUF backend) |

---

## When to Use This Skill

- Running LLMs locally on consumer hardware (CPU, Apple Silicon, GPU)
- Model discovery and selection from Hugging Face Hub
- Quantization for memory-constrained deployment
- High-throughput production model serving
- Guaranteed structured output (JSON, XML, code, regex)
- Multi-step constrained generation workflows
- Removing refusal behavior while preserving quality
- Edge deployment, offline inference, privacy-sensitive workloads