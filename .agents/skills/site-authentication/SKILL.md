---
name: site-authentication
description: 对需要登陆的页面，如何处理认证流程
---

# 通过统一认证登录业务页面

大部分南京大学网站支持统一认证登录。统一认证的网站是 authserver.nju.edu.cn.

一般来说，登录流程是：

1. 用户访问业务页面 https://ehall.nju.edu.cn/appShow?appId=4770397878132218
2. 网站发现没有登陆态cookie，跳转到统一认证 https://authserver.nju.edu.cn/authserver/login?service=https%3A%2F%2Fehall.nju.edu.cn%3A443%2Flogin%3Fservice%3Dhttps%3A%2F%2Fehall.nju.edu.cn%2FappShow%3FappId%3D4770397878132218
3. 一般来说，用户会在authserver页面输入用户名和密码来登录。但是，如果我们cookies里带有crates/nju-unified-auth提供的CASTGC，则不需要额外操作，统一认证页面会知道我们已经登录过并自动放行。
4. 然后，统一认证页面会跳转会业务页面
5. 此时，一路上各种set cookie使我们拥有了业务页面的登陆态，各种接口都可以随便请求了。

因此，代码实现上，一般是：

- 在cookie中设置authserver的castgc，并允许自动redirect
- 访问业务页面，等它跳转完
- 拿着这个session（reqwest client）去请求想要的api
