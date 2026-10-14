import {defineConfig} from '@playwright/test';
export default defineConfig({testDir:'./browser',workers:1,use:{headless:true,launchOptions:process.env.CHROMIUM_PATH?{executablePath:process.env.CHROMIUM_PATH}:{}},reporter:'list'});
