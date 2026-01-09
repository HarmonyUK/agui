//! Performance benchmarks for the Stream Timeline
//!
//! Tests rendering and virtualization performance with large item counts.
//! Run with: cargo bench --bench stream_timeline_bench

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use agui_desktop::stream::{
    StreamContent, StreamItem, StreamState, StreamTimeline,
    UserMessage, AgentMessage, ReasoningBlock, ToolCallBlock, ToolCallStatus,
    PlanBlock, PlanItem, PlanItemStatus, StatusBlock,
    virtual_list::{VirtualList, VirtualListConfig},
};

/// Generate a mix of stream items for benchmarking
fn generate_items(count: usize) -> Vec<StreamItem> {
    let mut items = Vec::with_capacity(count);

    for i in 0..count {
        let content = match i % 7 {
            0 => StreamContent::UserMessage(
                UserMessage::new(format!("User message #{}", i))
                    .with_sender("User"),
            ),
            1 => StreamContent::AgentMessage(
                AgentMessage::new("claude", format!(
                    "Agent response #{}\n\nThis is a longer message with multiple paragraphs.\n\n\
                     ```rust\nfn example() {{\n    println!(\"Hello\");\n}}\n```",
                    i
                )).with_name("Claude"),
            ),
            2 => StreamContent::Reasoning(
                ReasoningBlock::new(format!(
                    "Thinking about task #{}...\n\
                     1. First consideration\n\
                     2. Second consideration\n\
                     3. Third consideration",
                    i
                )).with_summary(format!("Analyzing task #{}", i)),
            ),
            3 => StreamContent::ToolCall(ToolCallBlock {
                call_id: format!("tc-{}", i),
                tool_name: format!("Tool{}", i % 5),
                parameters: serde_json::json!({
                    "param1": "value1",
                    "param2": i,
                }),
                status: match i % 4 {
                    0 => ToolCallStatus::Pending,
                    1 => ToolCallStatus::Running,
                    2 => ToolCallStatus::Completed,
                    _ => ToolCallStatus::Failed,
                },
                result: if i % 4 == 2 {
                    Some(serde_json::json!({"success": true}))
                } else {
                    None
                },
                error: if i % 4 == 3 {
                    Some("Error occurred".to_string())
                } else {
                    None
                },
                duration_ms: Some((i * 10) as u64),
                progress: if i % 4 == 1 { Some((i % 100) as u8) } else { None },
                expanded: false,
            }),
            4 => StreamContent::Plan(
                PlanBlock::new(format!("Plan #{}", i))
                    .with_items(vec![
                        PlanItem {
                            id: format!("p{}-1", i),
                            description: "First step".to_string(),
                            status: PlanItemStatus::Completed,
                            children: vec![],
                        },
                        PlanItem {
                            id: format!("p{}-2", i),
                            description: "Second step".to_string(),
                            status: PlanItemStatus::InProgress,
                            children: vec![],
                        },
                        PlanItem::new(format!("p{}-3", i), "Third step"),
                    ]),
            ),
            5 => StreamContent::StatusUpdate(
                StatusBlock::progress(format!("Processing item #{}...", i), (i % 100) as u8),
            ),
            _ => StreamContent::Divider,
        };

        items.push(StreamItem::new(format!("item-{}", i), content));
    }

    items
}

/// Benchmark adding items to stream state
fn bench_stream_state_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("stream_state_push");

    for count in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                let items = generate_items(count);
                b.iter(|| {
                    let mut state = StreamState::new();
                    for item in items.iter().cloned() {
                        state.push(black_box(item));
                    }
                    state
                });
            },
        );
    }

    group.finish();
}

/// Benchmark visible range calculation
fn bench_visible_range(c: &mut Criterion) {
    let mut group = c.benchmark_group("visible_range");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                let mut state = StreamState::new();
                state.set_viewport_height(600.0);
                for item in generate_items(count) {
                    state.push(item);
                }

                b.iter(|| {
                    // Scroll to middle
                    state.set_scroll_offset(state.total_height() / 2.0);
                    black_box(state.visible_range())
                });
            },
        );
    }

    group.finish();
}

/// Benchmark virtual list operations
fn bench_virtual_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("virtual_list");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                let config = VirtualListConfig::default();
                let mut list = VirtualList::new(config);
                list.set_viewport_height(600.0);
                list.set_item_count(count);

                // Set variable heights
                for i in 0..count {
                    list.set_item_height(i, 50.0 + (i % 100) as f32);
                }

                b.iter(|| {
                    // Simulate scrolling
                    list.scroll_by(1.0);
                    let range = list.visible_range();
                    black_box(range)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark item height estimation
fn bench_height_estimation(c: &mut Criterion) {
    let mut group = c.benchmark_group("height_estimation");

    let items = generate_items(1000);

    group.bench_function("estimate_heights", |b| {
        b.iter(|| {
            let mut total = 0.0f32;
            for item in &items {
                total += black_box(item.estimated_height());
            }
            total
        });
    });

    group.finish();
}

/// Benchmark scroll to item
fn bench_scroll_to_item(c: &mut Criterion) {
    let mut group = c.benchmark_group("scroll_to_item");

    for count in [1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                let config = VirtualListConfig::default();
                let mut list = VirtualList::new(config);
                list.set_viewport_height(600.0);
                list.set_item_count(count);

                // Set heights
                for i in 0..count {
                    list.set_item_height(i, 80.0);
                }

                let target = count / 2;

                b.iter(|| {
                    list.scroll_to_item(black_box(target));
                    list.visible_range()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark stream timeline with many items (low-end profile)
fn bench_low_end_profile(c: &mut Criterion) {
    let mut group = c.benchmark_group("low_end_profile");

    // Simulate low-end device with smaller viewport
    let viewport_height = 400.0;

    for count in [500, 1000, 2000].iter() {
        group.bench_with_input(
            BenchmarkId::new("items", count),
            count,
            |b, &count| {
                let mut timeline = StreamTimeline::new();
                timeline.set_viewport_height(viewport_height);

                for item in generate_items(count) {
                    timeline.push(item);
                }

                b.iter(|| {
                    // Simulate scrolling through the timeline
                    for _ in 0..10 {
                        timeline.scroll_by(1.0);
                        let range = timeline.virtual_list.visible_range();
                        black_box(range);
                    }

                    // Scroll to bottom
                    timeline.scroll_to_bottom();
                    black_box(timeline.virtual_list.is_at_bottom())
                });
            },
        );
    }

    group.finish();
}

/// Benchmark state updates (tool call status, etc.)
fn bench_state_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_updates");

    let mut state = StreamState::new();
    for item in generate_items(1000) {
        state.push(item);
    }

    group.bench_function("update_tool_call_status", |b| {
        b.iter(|| {
            // Update tool call statuses
            for i in (0..1000).step_by(7) {
                if i % 7 == 3 {
                    state.update_tool_call_status(
                        &format!("tc-{}", i),
                        ToolCallStatus::Completed,
                        None,
                        None,
                    );
                }
            }
        });
    });

    group.bench_function("toggle_expanded", |b| {
        b.iter(|| {
            // Toggle expansion on reasoning blocks
            for i in (0..1000).step_by(7) {
                if i % 7 == 2 {
                    state.toggle_expanded(&format!("item-{}", i));
                }
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_stream_state_push,
    bench_visible_range,
    bench_virtual_list,
    bench_height_estimation,
    bench_scroll_to_item,
    bench_low_end_profile,
    bench_state_updates,
);

criterion_main!(benches);
