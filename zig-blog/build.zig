const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // 创建可执行文件
    const exe = b.addExecutable(.{
        .name = "zig-blog",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    // 添加 SQLite 系统库链接
    exe.linkSystemLibrary("sqlite3");

    // 安装可执行文件
    b.installArtifact(exe);

    // 运行步骤
    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());

    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const run_step = b.step("run", "Run the zig-blog server");
    run_step.dependOn(&run_cmd.step);

    // 测试步骤
    const unit_tests = b.addTest(.{
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    unit_tests.linkSystemLibrary("sqlite3");

    const run_unit_tests = b.addRunArtifact(unit_tests);
    const test_step = b.step("test", "Run unit tests");
    test_step.dependOn(&run_unit_tests.step);
}