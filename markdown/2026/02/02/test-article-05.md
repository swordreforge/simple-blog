# 测试文章 05 - Git 工作流最佳实践

Git 是分布式版本控制系统，掌握其工作流对团队协作至关重要。

## 常用工作流

### Git Flow

- master/main：生产环境
- develop：开发环境
- feature/*：功能分支
- release/*：发布分支
- hotfix/*：紧急修复

### GitHub Flow

- main：生产环境
- feature/*：功能分支
- 通过 Pull Request 合并

## 最佳实践

1. **提交信息规范**：使用清晰的提交信息
2. **分支策略**：为每个功能创建独立分支
3. **代码审查**：通过 PR 进行代码审查
4. **持续集成**：自动化测试和部署

## 常用命令

```bash
git add .
git commit -m "feat: 添加新功能"
git push origin feature/new-feature
```

标签：git, 版本控制, 团队协作, devops
分类：技术
摘要：Git 是分布式版本控制系统，掌握其工作流对团队协作至关重要，本文介绍了 Git Flow 和 GitHub Flow 等工作流。
封面：/img/passage-cover.webp