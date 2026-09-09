import sys
import json
from .docarmor import DocumentAnalyzer

def run_mcp_server(analyzer=None):
    """
    Starts the native DocArmor Model Context Protocol (MCP) server over standard I/O (STDIO).
    Compatible with Claude Desktop, Cursor, Antigravity, Windsurf, and custom AI agents.
    """
    analyzer = analyzer or DocumentAnalyzer()
    analyzer.run_mcp_server()

def main():
    try:
        run_mcp_server()
    except KeyboardInterrupt:
        sys.exit(0)

if __name__ == "__main__":
    main()
