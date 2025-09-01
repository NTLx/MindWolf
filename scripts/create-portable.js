import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// 源文件路径
const sourceExe = path.join(__dirname, '..', 'src-tauri', 'target', 'x86_64-pc-windows-msvc', 'release', 'mindwolf.exe');

// 目标目录
const portableDir = path.join(__dirname, '..', 'portable');
const targetExe = path.join(portableDir, 'mindwolf.exe');

// 创建便携式目录
if (!fs.existsSync(portableDir)) {
    fs.mkdirSync(portableDir, { recursive: true });
}

// 复制可执行文件
console.log('创建便携式版本...');
fs.copyFileSync(sourceExe, targetExe);

// 创建说明文件
const readmeContent = `# 智狼 (MindWolf) - 便携版

这是智狼 (MindWolf) AI狼人杀游戏的便携版本。

## 使用方法

1. 双击 mindwolf.exe 启动应用
2. 首次运行会在当前目录下创建 config 目录存储配置
3. 如需重置配置，删除 config 目录即可

## 系统要求

- Windows 10 或更高版本
- x64 架构

## 文件说明

- mindwolf.exe: 主程序文件 (约 ${Math.round(fs.statSync(targetExe).size / 1024 / 1024 * 10) / 10} MB)
- config/: 配置文件目录 (首次运行后自动创建)
- logs/: 日志文件目录 (首次运行后自动创建)

## 版本信息

构建时间: ${new Date().toLocaleString('zh-CN')}
版本: 0.1.0
`;

fs.writeFileSync(path.join(portableDir, 'README.md'), readmeContent, 'utf8');

console.log(`✅ 便携版本创建完成！`);
console.log(`📁 位置: ${portableDir}`);
console.log(`📦 大小: ${Math.round(fs.statSync(targetExe).size / 1024 / 1024 * 10) / 10} MB`);
console.log(`🚀 运行: ${targetExe}`);