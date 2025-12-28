<script setup lang="ts">
import {onMounted, reactive, ref} from "vue";
import {message} from "ant-design-vue";
import type {FormInstance} from "ant-design-vue";
import {getCaptcha,registerApi} from "../api/auth.ts";
import {useRouter} from "vue-router";

//注册表单类型定义
type RegisterForm = {
  username: string;
  email: string;
  password: string;
  confirmPassword: string;
  captcha: string;
  captchaId: string;
};

const router = useRouter();

//把表单对象变成响应式数据
const form = reactive<RegisterForm>({
  username: "",
  email: "",
  password: "",
  confirmPassword: "",
  captcha: "",
  captchaId:""
});

//判断是否正在注册(占位）
//后面接入axios请求后，请求开始loading=true，请求结束loading=false
const loading = ref(false);

//验证码图片占位
const captchaImgSrc = ref("");

//点击验证码刷新占位
async function refreshCaptchaImg() {
  try {
    const data=await getCaptcha();
    captchaImgSrc.value=data.image;
    form.captchaId=data.captcha_id;
    form.captcha="";
  }catch(e:any){
    message.error(e?.response?.data?.message||"验证码加载失败");
  }
}

onMounted(() => {
  refreshCaptchaImg()
});

//密码格式前端校验
const passwordRules = [
  {required: true, message: "请输入密码", trigger: ["change", "blur"]},
  {
    pattern: /^(?=.*[A-Z])(?=.*[a-z])(?=.*\d)[A-Za-z0-9!@#$%^&*()_+.,]{6,}$/,
    message: "密码需要大于6位，且必须包含大写字母、小写字母和数字，符号仅限!@#$%^&*()_+,.",
    trigger: ["change", "blur"]
  }
]

//确认密码前端校验
const confirmPasswordRulesValidator = async (_rule: any, values: string) => {
  if (!values) return Promise.reject("请再次输入密码");
  if (values !== form.password) return Promise.reject("两次输入的密码不一致");
  return Promise.resolve();
}

//全部校验
const rules = {
  username: [{required: true, message: "请输入用户名", trigger: ["change", "blur"]}],
  email: [
    {required: true, message: "请输入邮箱", trigger: ["change", "blur"]},
    {type: "email", message: "邮箱格式不正确", trigger: ["change", "blur"]},
  ],
  password: passwordRules,
  confirmPassword: [{validator: confirmPasswordRulesValidator, trigger: ["change", "blur"]}],
  captcha: [{required: true, message: "请输入验证码", trigger: ["change", "blur"]}],
}

//注册按钮，模拟成功
const formRef = ref<FormInstance>();

async function onRegisterClick() {
  if (!form.captchaId) {
    message.warning("验证码未加载，请刷新验证码");
    return;
  }
  try{
    //先进行基本检测
    await formRef.value?.validate();
  }catch (e){
    message.error("提交的表单有误，请按下方红色提示修改");
    return;
  }
  if(form.password!==form.confirmPassword){
    message.warning("两次输入的密码不一致");
    return;
  }

  //通过上方的校验后可以走到这一步
  loading.value = true;
  try {
     await registerApi({
      username: form.username,
      email: form.email,
      password: form.password,
      captcha: form.captcha,
      captcha_id: form.captchaId,
    });

  message.success("注册成功");
  router.push("/login");
  }catch(e:any){
    message.error(e?.response?.data?.message||e?.response?.data||"注册失败");
    await refreshCaptchaImg();
  }finally {
    loading.value=false;
  }
}
</script>

<template>
  <!--  最外层容器-->
  <div class="auth-page">
    <!--    内容卡片-->
    <a-card class="auth-card" :bordered="false">
      <!--      容器拆分左右两份-->
      <div class="auth-card-content">
        <!--        左侧Logo部分-->
        <div class="auth-card-left">
          <img src="@/assets/logo.png" alt="logo" class="logo"/>
        </div>

        <!--        右侧部分-->
        <div class="auth-card-right">
          <!--          大标题-->
          <div class="form-title">账号注册</div>

          <div class="form-control">
            <!--            表单内容-->
            <a-form :model="form" layout="vertical" :rules="rules" :ref="formRef">
              <!--              用户名-->
              <a-form-item class="register-item" name="username">
                <div class="field-row">
                  <div class="field-label">用户名</div>
                  <div class="field-control">
                    <a-input
                        v-model:value="form.username"
                        placeholder="请输入用户名"
                        autocomplete="username"
                        allow-clear/>
                  </div>
                </div>
              </a-form-item>
              <!--              邮箱-->
              <a-form-item class="register-item" name="email">
                <div class="field-row">
                  <div class="field-label">邮箱</div>
                  <div class="field-control">
                    <a-input
                        v-model:value="form.email"
                        placeholder="请输入邮箱"
                        autocomplete="email"
                        allow-clear/>
                  </div>
                </div>
              </a-form-item>
              <!--              密码-->
              <a-form-item class="register-item" name="password">
                <div class="field-row">
                  <div class="field-label">密码</div>
                  <div class="field-control">
                    <a-input-password
                        v-model:value="form.password"
                        placeholder="请输入密码"
                        autocomplete="new-password"
                        allow-clear/>
                  </div>
                </div>
              </a-form-item>
              <!--              确认密码-->
              <a-form-item class="register-item" name="confirmPassword">
                <div class="field-row">
                  <div class="field-label">确认</div>
                  <div class="field-control">
                    <a-input-password
                        v-model:value="form.confirmPassword"
                        placeholder="请再次输入密码"
                        autocomplete="new-password"
                        allow-clear/>
                  </div>
                </div>
              </a-form-item>
              <!--              验证码-->
              <a-form-item class="register-item" name="captcha">
                <div class="captcha-spilt">
                  <!--                  左侧标题+输入框-->
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
                  <!--                  右侧验证码部分-->
                  <img
                      class="captcha-img"
                      :src="captchaImgSrc"
                      alt="captcha"
                      @click="refreshCaptchaImg"/>
                </div>
              </a-form-item>
              <!--              提交注册-->
              <a-form-item class="register-submit">
                <a-button
                    class="login-submit-btn"
                    type="primary"
                    :loading="loading"
                    @click="onRegisterClick"
                    block>
                  注册
                </a-button>
              </a-form-item>
              <!--              登录-->
              <a-form-item class="login">
                <span>已有账号？点击</span>
                <router-link class="login-link" to="/login">登录</router-link>
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
  padding: 8px;
  width: 420px;
  max-width: 100%;
}

/*标题*/
.form-title {
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

/*验证码输入框 + 图片一行显示*/
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

/*按钮间距*/
.register-submit {
  padding-top: 10px;
  margin-bottom: 10px;
}

/*底部“已有账户”*/
.login {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 10px;
}

</style>