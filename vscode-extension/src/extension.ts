import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import { execFile } from 'child_process';

export function activate(context: vscode.ExtensionContext) {
    // 1. Status Bar Item
    const statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBar.text = "$(zap) ezContext: Active";
    statusBar.tooltip = "ezContext AST Token Saver MCP is ready. Click to setup in AI tools.";
    statusBar.command = "ezcontext.setupMcp";
    statusBar.show();
    context.subscriptions.push(statusBar);

    // 2. Command: 1-Click MCP Setup in Cline & Roo Code
    const setupDisposable = vscode.commands.registerCommand('ezcontext.setupMcp', async () => {
        try {
            const homeDir = os.homedir();
            // Standard config locations for Cline and Roo Code
            const configPaths = [
                path.join(homeDir, '.config', 'Code', 'User', 'globalStorage', 'saoudrizwan.claude-dev', 'settings', 'cline_mcp_settings.json'),
                path.join(homeDir, '.config', 'Code', 'User', 'globalStorage', 'rooveterinaryinc.roo-cline', 'settings', 'cline_mcp_settings.json'),
            ];

            let configuredCount = 0;
            for (const confPath of configPaths) {
                const dir = path.dirname(confPath);
                if (fs.existsSync(dir)) {
                    let settings: any = { mcpServers: {} };
                    if (fs.existsSync(confPath)) {
                        try {
                            const raw = fs.readFileSync(confPath, 'utf8');
                            settings = JSON.parse(raw);
                            if (!settings.mcpServers) {
                                settings.mcpServers = {};
                            }
                        } catch {
                            settings = { mcpServers: {} };
                        }
                    }

                    settings.mcpServers["ezcontext"] = {
                        command: "ezcontext",
                        args: ["serve"],
                        disabled: false,
                        autoApprove: ["get_skeleton", "get_outline", "read_symbol"]
                    };

                    fs.writeFileSync(confPath, JSON.stringify(settings, null, 2), 'utf8');
                    configuredCount++;
                }
            }

            if (configuredCount > 0) {
                vscode.window.showInformationMessage(`⚡ ezContext MCP configured in ${configuredCount} AI Assistant(s)! Your AI agents will now automatically use AST skeletons to save up to 90% tokens.`);
            } else {
                // Copy manual snippet to clipboard
                const manualConfig = JSON.stringify({
                    "mcpServers": {
                        "ezcontext": {
                            "command": "ezcontext",
                            "args": ["serve"],
                            "disabled": false,
                            "autoApprove": ["get_skeleton", "get_outline", "read_symbol"]
                        }
                    }
                }, null, 2);
                await vscode.env.clipboard.writeText(manualConfig);
                vscode.window.showInformationMessage("ezContext MCP JSON configuration copied to clipboard! Paste it into your AI assistant's MCP settings.");
            }
        } catch (err: any) {
            vscode.window.showErrorMessage(`Failed to configure ezContext: ${err.message}`);
        }
    });
    context.subscriptions.push(setupDisposable);

    // 3. Command: Copy Compact Skeleton to Clipboard
    const copyDisposable = vscode.commands.registerCommand('ezcontext.copySkeleton', async (uri?: vscode.Uri) => {
        const targetUri = uri || vscode.window.activeTextEditor?.document.uri;
        if (!targetUri) {
            vscode.window.showWarningMessage("Please select a file to extract skeleton.");
            return;
        }

        const filePath = targetUri.fsPath;
        execFile('ezcontext', ['skeleton', filePath], (error, stdout, _stderr) => {
            if (error) {
                // Fallback to reading file and notifying
                vscode.window.showErrorMessage(`ezcontext binary error: ${error.message}. Ensure 'ezcontext' is in your PATH.`);
                return;
            }

            vscode.env.clipboard.writeText(stdout);
            vscode.window.showInformationMessage(`📋 Copied AST Code Skeleton for ${path.basename(filePath)} to clipboard! Ready to paste into ChatGPT/Claude.`);
        });
    });
    context.subscriptions.push(copyDisposable);
}

export function deactivate() {}
