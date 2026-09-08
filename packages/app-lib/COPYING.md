# Copying

WispLauncher's backend is a modified version of Modrinth App's backend. It is licensed under the GNU General Public License, Version 3 only, provided in [LICENSE](./LICENSE).

Copyright for the original work remains with Rinth, Inc. and the original contributors. Wisp modifications are Copyright © 2026 Garbage Human Studio and were developed by Mystic Stars.

WispLauncher is an independent, unofficial client. Modrinth is a trademark of Rinth, Inc. and is referenced only to identify API and file-format compatibility.WispLauncher is not affiliated with or endorsed by Rinth, Inc.

## AI integration attribution

The AI provider catalog and provider defaults are independently adapted from [LobeChat](https://github.com/lobehub/lobe-chat) at commit `a27dfaeda1ab499ac024a6eb0448917b216ba8a1`. Bundled text-model metadata is synchronized from [LobeHub's model bank](https://github.com/lobehub/lobehub/tree/main/packages/model-bank/src/aiModels); the exact source revision is recorded in `src/api/lobehub_text_models.json`. LobeChat is distributed under the LobeHub Community License; a verbatim copy of that license is provided in [third-party/licenses/LobeHub-Community-License.txt](../../third-party/licenses/LobeHub-Community-License.txt). Provider protocol clients in this package are GPL-compatible implementations against the providers' documented wire formats; LobeHub-licensed application source is not incorporated into this GPL backend.

The AI translation configuration flow is adapted from [Read Frog](https://github.com/mengxi-ream/read-frog) at commit `951e20fa82cd96d606a44156362f402c79d11473`. Read Frog is distributed under the GNU General Public License, Version 3; a copy is provided in [third-party/licenses/Read-Frog-GPL-3.0.txt](../../third-party/licenses/Read-Frog-GPL-3.0.txt).

The Google Translate direct-connect IP list is sourced from [Ponderfly/GoogleTranslateIpCheck](https://github.com/Ponderfly/GoogleTranslateIpCheck), distributed under the MIT License; a copy is provided in [third-party/licenses/MIT.txt](../../third-party/licenses/MIT.txt).
