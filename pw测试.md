
  1) tests/route-fields-comprehensive.spec.js:375:5 › 动态路由全面测试套件 › 边界条件测试 › 空字段验证 ────────────────

    Error: expect(received).toBe(expected) // Object.is equality

    Expected: true
    Received: false

      385 |         return errors.length > 0;
      386 |       });
    > 387 |       expect(hasError).toBe(true);
          |                        ^
      388 |     });
      389 |
      390 |     test('极长字段测试', async ({ page }) => {
        at /home/swordreforge/projects/rustblog-new/rustblog/tests/route-fields-comprehensive.spec.js:387:24

    Error Context: test-results/tests-route-fields-comprehensive-动态路由全面测试套件-边界条件测试-空字段验证/error-context.md

  2) tests/route-fields-comprehensive.spec.js:492:5 › 动态路由全面测试套件 › 删除路由测试 › 删除单个路由 ───────────────

    Error: expect(received).toBe(expected) // Object.is equality

    Expected: false
    Received: true

      511 |
      512 |       const exists = await checkRouteExists(page, routeId);
    > 513 |       expect(exists).toBe(false);
          |                      ^
      514 |     });
      515 |
      516 |     test('批量删除路由', async ({ page }) => {
        at /home/swordreforge/projects/rustblog-new/rustblog/tests/route-fields-comprehensive.spec.js:513:22

    Error Context: test-results/tests-route-fields-comprehensive-动态路由全面测试套件-删除路由测试-删除单个路由/error-context.md

  3) tests/route-fields-comprehensive.spec.js:578:5 › 动态路由全面测试套件 › 数据一致性测试 › 存储统计准确性 ─────────────

    Error: expect(received).toBe(expected) // Object.is equality

    Expected: 5
    Received: 7

      615 |
      616 |       // 验证统计准确性
    > 617 |       expect(updatedStats.database).toBe(initialStats.database + 1);
          |                                     ^
      618 |       expect(updatedStats.memory).toBe(initialStats.memory + 1);
      619 |       expect(updatedStats.file).toBe(initialStats.file + 1);
      620 |       expect(updatedStats.total).toBe(initialStats.total + 3);
        at /home/swordreforge/projects/rustblog-new/rustblog/tests/route-fields-comprehensive.spec.js:617:37

    Error Context: test-results/tests-route-fields-comprehensive-动态路由全面测试套件-数据一致性测试-存储统计准确性/error-context.md

  3 failed
    tests/route-fields-comprehensive.spec.js:375:5 › 动态路由全面测试套件 › 边界条件测试 › 空字段验证 ─────────────────
    tests/route-fields-comprehensive.spec.js:492:5 › 动态路由全面测试套件 › 删除路由测试 › 删除单个路由 ────────────────
    tests/route-fields-comprehensive.spec.js:578:5 › 动态路由全面测试套件 › 数据一致性测试 › 存储统计准确性 ──────────────
  27 passed (3.0m)
