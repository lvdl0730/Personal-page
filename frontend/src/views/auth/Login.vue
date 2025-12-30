<script setup lang="ts">
import {reactive, ref, onMounted} from 'vue';
import {message} from "ant-design-vue";
import { useRouter} from "vue-router";
import {getCaptcha, loginApi} from "@/api/auth";
import {useAuthStore} from "@/stores/auth";

//登录表单类型定义
type LoginForm = {
  account: string;
  password: string;
  captcha: string;
  captchaId: string;
  remember: boolean;
}

const router = useRouter();
const auth = useAuthStore();

//把表单对象变成响应式是数据
const form = reactive<LoginForm>({
  account: "",
  password: "",
  captcha: "",
  captchaId: "",
  remember: true,
});

//判断是否正在登录（占位）
const loading = ref(false);

//验证码图片占位
const captchaImgSrc = ref("");

//点击验证码刷新:向后端请求captcha_id+image
async function refreshCaptchaImg() {
  try {
    const data = await getCaptcha();
    captchaImgSrc.value = data.image;
    form.captchaId = data.captcha_id;
    form.captcha = "";//刷新验证码后把输入框清空
  } catch (e: any) {
    message.error(e?.response?.data?.message || "验证码获取失败");
  }
}

onMounted(() => {
  refreshCaptchaImg()
});

//登录按钮：校验->请求->保存token->跳转
async function onLoginClick() {
  //完整性验证
  if (!form.account || !form.password || !form.captcha) {
    message.warning("请填写完整信息");
    return;
  }
  if (!form.captchaId) {
    message.warning("验证码未加载，请刷新验证码");
    return;
  }

  //模拟正在登录
  loading.value = true;
  try {
    const resp = await loginApi({
      account: form.account,
      password: form.password,
      captcha_id: form.captchaId,
      captcha: form.captcha,
    });

    //把token交给store管理，同时按照remember的值存储
    auth.setToken(resp.token, form.remember);

    message.success("登录成功");
    router.push("/dashboard");
  } catch (e: any) {
    message.error(e?.response?.data?.message || e?.response?.data || "登录失败");
    await refreshCaptchaImg();
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <!--最外层容器-->
  <div class="auth-page">
    <!--    内容卡片-->
    <a-card class="auth-card" :bordered="false">
      <!--      容器拆成左右两份-->
      <div class="auth-card-content">

        <!--        左侧Logo部分-->
        <div class="auth-card-left">
          <img src="@/assets/logo.png" alt="logo" class="logo"/>
        </div>

        <!--        右侧部分-->
        <div class="auth-card-right">
          <!--          大标题-->
          <div class="from-title">账号登录</div>
          <div class="form-control">
            <!--          表单内容-->
            <a-form :model="form" layout="vertical">
              <!--            用户名+邮箱-->
              <a-form-item class="login-item" name="account">
                <div class="field-row">
                  <div class="field-label">用户名</div>
                  <div class="field-control">
                    <a-input
                        v-model:value="form.account"
                        placeholder="请输入用户名或邮箱"
                        autocomplete="username"
                        allow-clear/>
                  </div>
                </div>
              </a-form-item>

              <!--            密码-->
              <a-form-item class="login-item" name="password">
                <div class="field-row">
                  <div class="field-label">密码</div>
                  <div class="field-control">
                    <a-input-password
                        v-model:value="form.password"
                        placeholder="请输入密码"
                        autocomplete="password"
                        allow-clear/>
                  </div>
                </div>
              </a-form-item>

              <!--            验证码+图片-->
              <a-form-item class="login-item" name="captcha">
                <div class="captcha-spilt">
                  <!--              左半部分标题+输入框-->
                  <div class="field-row">
                    <div class="field-label">验证码</div>
                    <div class="field-control">
                      <a-input
                          class="captcha-input"
                          v-model:value="form.captcha"
                          placeholder="请输入验证码"
                          autocomplete="captcha"
                          allow-clear/>
                    </div>
                  </div>
                  <!--              右半部分验证码图片-->
                  <img
                      class="captcha-img"
                      :src="captchaImgSrc"
                      alt="captcha"
                      @click="refreshCaptchaImg"/>
                </div>
              </a-form-item>

              <!--            记住密码+忘记密码-->
              <a-row class="login-link-row">
                <!--              记住密码-->
                <a-checkbox v-model:value="form.remember">记住密码</a-checkbox>
                <!--              忘记密码-->
                <router-link class="login-forget" to="/forget">忘记密码</router-link>
              </a-row>

              <!--            登录按钮，:loading="loading以后接入API时显示正在登录-->
              <a-form-item class="login-submit" name="remember">
                <a-button
                    class="login-submit-btn"
                    type="primary"
                    :loading="loading"
                    @click="onLoginClick"
                    block>
                  登录
                </a-button>
              </a-form-item>

              <!--            点击注册-->
              <a-form-item class="register">
                <span>没有账号？点击</span>
                <router-link class="login-link" to="/register">注册</router-link>
              </a-form-item>
            </a-form>
          </div>
        </div>
      </div>
    </a-card>
  </div>
</template>

<style scoped>
/*页面背景+整体居中*/
.auth-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

/*内容卡片*/
.auth-card {
  width: 920px;
  max-width: 90%;
  border-radius: 10px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.32);
}

/*左右布局*/
.auth-card-content {
  display: flex;
  gap: 24px;
}

/*左边部分*/
.auth-card-left {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px 8px;
  border-right: 1px solid black;
}

/*logo*/
.logo {
  max-width: 420px;
  object-fit: contain;
}

/*右侧部分*/
.auth-card-right {
  flex: 1;
  padding: 24px 8px;
  width: 420px;
  max-width: 100%;
}

/*标题*/
.from-title {
  display: flex;
  font-weight: 600;
  font-size: 30px;
  justify-content: center;
  padding-top: 20px;
  padding-bottom: 50px;
}

/*输入行整体*/
.field-row {
  height: 48px;
  display: flex;
  align-items: center;
  gap: 10px;
  border-radius: 20px;
  background: white;
  border: 1px solid rgba(128, 128, 128, 0.34);
  padding: 0 12px;
}

/*输入行标题*/
.field-label {
  width: 60px;
  flex: 0 0 auto;
  color: gray;
  font-size: 16px;
  font-weight: 600;
  text-align: center;
}

/*调整输入框*/
/*调整输入框边框透明*/
.field-row :deep(.ant-input),
.field-row :deep(.ant-input-affix-wrapper) {
  border: none !important;
  background: transparent !important;
  width: 100%;
  flex: 1;
}

/*设置输入框高度*/
.field-row :deep(.ant-input) {
  height: 36px;
  padding: 0;
}

/*输入框长度占满剩余位置*/
.field-control {
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 0;
}

/*单独调整验证码输入框*/
.captcha-spilt {
  display: flex;
  align-items: center;
  gap: 10px;
}

.captcha-input {
  gap: 10px;
  flex: 1;
  min-width: 0;
}

.captcha-img {
  width: 120px;
  height: 42px;
}

/*记住密码+忘记密码*/
.login-link-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-right: 15px;
  padding-left: 15px;
  padding-bottom: 5px;
}

.login-submit {
  padding-top: 10px;
  margin-bottom: 10px;
}

.register {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 10px;
}

</style>