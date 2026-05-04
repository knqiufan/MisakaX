import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import enCommon from "./en/common.json";
import enNav from "./en/nav.json";
import enSettings from "./en/settings.json";

import zhCommon from "./zh-CN/common.json";
import zhNav from "./zh-CN/nav.json";
import zhSettings from "./zh-CN/settings.json";

const resources = {
  en: {
    common: enCommon,
    nav: enNav,
    settings: enSettings,
  },
  "zh-CN": {
    common: zhCommon,
    nav: zhNav,
    settings: zhSettings,
  },
};

i18n.use(initReactI18next).init({
  resources,
  lng: "en",
  fallbackLng: "en",
  defaultNS: "common",
  ns: ["common", "nav", "settings"],
  interpolation: {
    escapeValue: false,
  },
});

export { i18n };
