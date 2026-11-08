#!/usr/bin/env python3
"""Generate the Chinese learning pages from the translated source below.

The explanatory text and referenced examples are translated/adapted from the
user-supplied Sudoku 401 help directory. Run from any working directory.
"""
from pathlib import Path
from html import escape
import shutil

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT.parent / 'analysis/ipa/Payload/Sudoku 401.app/help'
OUT = ROOT / 'help'
# Paragraphs are trusted local HTML, so cross references remain relative links.
PAGES = {
'full_house': ('唯一空格', 'Full House', [
'这是最基础的数独步骤。一个单元（行、列或宫）已有 8 个确定数字时，唯一的空格只能填入缺少的数字。',
'在下图中，第 2 行已有 8 个数字，只有 R2C8 为空，因此 R2C8 只能填 6。', '@full_house_1.png']),
'naked_single': ('显性单数', 'Naked Single', [
'当某个空格只有一个候选数时，它就是显性单数。',
'下图中，第 9 列已有 1、2、3、5，第 3 行已有 5、8，第 3 宫已有 3、4、6、7，因此 R3C9 只能填 9。寻找显性单数往往要同时观察该格所在的行、列和宫。', '@naked_single_1.png']),
'hidden_single': ('隐性单数', 'Hidden Single', [
'在一个单元（行、列或宫）中，若某个数字只有一个可能的位置，这个位置就是隐性单数。',
'下图中，第 1 宫的第 2、3 列以及第 3 行都不能填 9，所以 9 只能放在 R2C1。', '@hidden_single_1.png',
'更容易发现隐性单数的方法是点击一个已知数字。Sudobility 的 iPhone 软件会自动将无法填入该数字的格子变灰。下图中点击数字 4 后，可以看出第 2 宫的 4 只能位于 R1C6。', '@hidden_single_2.png']),
'locked_set': ('锁定数组', 'Locked Set', [
'锁定数组是同一单元中 N 个格子的候选数合起来恰好只有 N 种。',
'最简单的锁定数组是一个格子只有一个候选数，即<a href="naked_single.html">显性单数</a>。两个格子含两个候选数称为<a href="naked_pair.html">显性数对</a>；三个格子含三个候选数称为<a href="naked_triple.html">显性三数组</a>；四个格子含四个候选数称为<a href="naked_quad.html">显性四数组</a>。这些候选数可从同一单元内其他格子删除。']),
'naked_pair': ('显性数对', 'Naked Pair', [
'同一行、列或宫中，如果两个格子都只含相同的两个候选数，就可以从该单元的其他格子中删除这两个候选数。',
'下图中，第 8 列的 R2C8 和 R5C8 都只有候选 2、6。无论哪个格子填 2，另一个都必须填 6，因此 R9C8 不能填 2 或 6。', '@naked_pair_1.png']),
'hidden_pair': ('隐性数对', 'Hidden Pair', [
'隐性数对用于删除候选数，需要先记录草稿。在一个行、列或宫中，如果两个数字只可能出现在同样的两个格子里，就可以删除这两个格子的其他候选数。',
'下图第 9 行的候选 5、8 只能出现在 R9C4 和 R9C9，二者必定分别占据这两个格子，因此可从 R9C9 删除 3、4、6、7。', '@hidden_pair_1.png']),
'locked_candidates': ('区块摒除', 'Locked Candidates', [
'这是依靠单个数字删除候选的技巧，需要先记录草稿。如果数字在一个单元中只能出现在它与另一个单元的交集，就可以从第二个单元的交集以外删除这个数字。有效的交集发生在一个宫和一行或一列之间。',
'下图中，第 6 宫的候选 2 只能位于该宫与第 9 列的交集：R4C9、R5C9 或 R6C9。因此 R7C9 不能填 2。', '@locked_candidates_1.png']),
'naked_triple': ('显性三数组', 'Naked Triple', [
'同一行、列或宫中，若三个格子的候选数合起来恰好是三个数字，且没有其他候选数，则这些数字可从该单元其他格子删除。这是<a href="naked_pair.html">显性数对</a>推广到三个格子的情况。',
'下图中，第 5 宫的 R5C4、R6C4、R6C6 合起来只有候选 2、3、7。因此可以从 R4C4、R4C5、R4C6 和 R5C5 删除 2、3、7。', '@naked_triple_1.png']),
'hidden_triple': ('隐性三数组', 'Hidden Triple', [
'隐性三数组将<a href="hidden_pair.html">隐性数对</a>推广到三个候选数和三个格子：同一行、列或宫中，若三个数字只出现在同样的三个格子里，可以删除这三个格子中的其他候选数。']),
'naked_quad': ('显性四数组', 'Naked Quad', [
'同一行、列或宫中，若四个格子的候选数合起来恰好是四个数字，且没有其他候选数，就可以从该单元其他格子删除这四个数字。这是<a href="naked_pair.html">显性数对</a>推广到四个格子的情况。']),
'hidden_quad': ('隐性四数组', 'Hidden Quad', [
'隐性四数组将<a href="hidden_pair.html">隐性数对</a>推广到四个候选数和四个格子：同一行、列或宫中，若四个数字只出现在同样的四个格子里，可以删除这四个格子中的其他候选数。']),
'x_wing': ('X 翼', 'X-Wing', [
'X 翼是利用单个数字删除候选的技巧，需要先记录草稿。如果某个数字在两行中的候选位置都只落在相同的两列，就可以从这两列的其他行删除该数字。行列互换也成立。',
'下图中，第 1、8 行的候选 7 都只在第 1、5 列。无论 R1C1 与 R8C5 为 7，还是 R1C5 与 R8C1 为 7，第 5 列的其他格子都不能为 7，因此可以从 R2C5 和 R9C5 删除 7。', '@x_wing_1.png']),
'swordfish': ('剑鱼', 'Swordfish', [
'剑鱼是需要草稿的单数字消除技巧，把<a href="x_wing.html">X 翼</a>推广到三行三列。若一个数字在三行中的所有候选位置都落在相同的三列，可从这三列的其他行删除该数字；行列互换同理。']),
'jellyfish': ('水母', 'Jellyfish', [
'水母是需要草稿的单数字消除技巧，把<a href="x_wing.html">X 翼</a>推广到四行四列。若一个数字在四行中的所有候选位置都落在相同的四列，可从这四列的其他行删除该数字；行列互换同理。']),
'squirmbag': ('五阶鱼', 'Squirmbag', [
'这是需要草稿的单数字消除技巧，可看作<a href="x_wing.html">X 翼</a>向五行五列或更多行列的推广。若一个数字在 N 行中的所有候选位置都落在相同的 N 列，可从这 N 列的其他行删除该数字；行列互换同理。']),
'finned_x_wing': ('带鳍 X 翼', 'Finned X-Wing', [
'带鳍 X 翼是带有鳍候选的 X 翼；可以删除同时受到鳍和 X 翼约束的候选。',
'下图中，若 R4C9 不是 2，第 1、4 行将形成数字 2 的 X 翼。无论是 X 翼成立，还是 R4C9 为 2，R6C8 都不能为 2。', '@finned_x_wing_1.png']),
'finned_swordfish': ('带鳍剑鱼', 'Finned Swordfish', [
'带鳍剑鱼是带有鳍候选的<a href="swordfish.html">剑鱼</a>；可删除同时受到鳍和鱼形约束的候选。鳍的基本推理可先阅读<a href="finned_x_wing.html">带鳍 X 翼</a>。',
'下图中，若 R9C8 不是 8，第 1、8、9 列将形成数字 8 的剑鱼。无论剑鱼成立，还是 R9C8 为 8，R7C7 都不能为 8。', '@finned_swordfish_1.png']),
'finned_jellyfish': ('带鳍水母', 'Finned Jellyfish', [
'带鳍水母是带有鳍候选的<a href="jellyfish.html">水母</a>；可删除同时受到鳍和鱼形约束的候选。请先阅读<a href="finned_x_wing.html">带鳍 X 翼</a>了解鳍的推理方式。']),
'finned_squirmbag': ('带鳍五阶鱼', 'Finned Squirmbag', [
'带鳍五阶鱼是带有鳍候选的<a href="squirmbag.html">五阶鱼</a>；可删除同时受到鳍和鱼形约束的候选。请先阅读<a href="finned_x_wing.html">带鳍 X 翼</a>了解鳍的推理方式。']),
'xy_wing': ('XY 翼', 'XY-Wing', [
'寻找三个双候选格，候选分别为 XZ、XY、YZ。第一个格子与第二个格子互相可见，第二个格子与第三个格子互相可见。随后找到同时看见第一个和第三个格子、且含候选 Z 的其他格子。',
'如果第一个格子是 Z，目标格不能是 Z；如果它是 X，则第二格为 Y、第三格为 Z，目标格同样不能是 Z。',
'下图中，R1C9、R2C7、R9C7 构成 XY 翼，其中 X=8、Y=3、Z=1。无论哪种情况，都可从 R3C7 和 R8C9 删除 1。', '@xy_wing_1.png',
'也可将 XY 翼视为最简单的<a href="als.html">ALS</a>情形：R1C9 是一个双候选单格 ALS；R2C7 与 R9C7 组成另一个 ALS。8 是受限共同候选，1 是可以从同时看见两端的格子中删除的共同候选。']),
'xyz_wing': ('XYZ 翼', 'XYZ-Wing', [
'XYZ 翼是<a href="xy_wing.html">XY 翼</a>的变体：三个格子的候选分别为 XZ、XYZ、YZ；中间格分别与两端互相可见。',
'可以从同时看见这三个结构格子的其他格子中删除候选 Z。']),
'wxyz_wing': ('WXYZ 翼', 'WXYZ-Wing', [
'WXYZ 翼可以视为<a href="als.html">ALS</a>的一种简化情况，比<a href="xy_wing.html">XY 翼</a>稍难发现。寻找一个双候选格，以及由三个格子组成的另一个 ALS。',
'下图中，蓝色双候选格和绿色 ALS 的受限共同候选是 9。候选 4 位于两个 ALS 的 R7C7 与 R5C9，可从同时看见这两格的 R6C7 和 R9C9 删除 4。', '@xyz_wing_1.png',
'如果 R7C7 为 4，两个目标格都不能为 4；如果 R7C7 为 9，绿色格子被迫成为 2、3、4 的显性三数组，两个目标格仍不能为 4。']),
'als': ('近似锁定数组', 'Almost Locked Sets (ALS)', [
'近似锁定数组（ALS）是同一单元中 N 个格子含有 N+1 种候选数字的集合；它比锁定数组多一个候选。',
'两个 ALS 的<strong>受限共同候选</strong>是不能同时出现在两个 ALS 中的共同候选。因此，如果该数字在其中一个 ALS 成立，另一个 ALS 就会变为锁定数组。',
'解题时寻找具有受限共同候选的两个 ALS，再寻找另一共同候选：若某个外部格子同时看见两个 ALS 中所有含这一候选的格子，就可以从该外部格删除它。',
'下图中绿色和蓝色格子分别组成一个 ALS，5 是受限共同候选。候选 1 位于绿色 ALS 的 R1C2 和蓝色 ALS 的 R4C6；R1C6 同时看见两者，因此可从 R1C6 删除 1。', '@als_1.png',
'如果 R1C2 为 1，R1C6 不能为 1；如果绿色 ALS 不含 1，它就锁定为 3、4、5，并迫使蓝色 ALS 锁定为 1、4、9，进而 R4C6 为 1，R1C6 仍不能为 1。']),
'als_chain': ('ALS 链', 'ALS-Chain', [
'请先理解<a href="als.html">近似锁定数组（ALS）</a>。ALS 链将这一概念扩展到两个以上的 ALS，并以受限共同候选依次连接。',
'沿链推理时，首端的取值会约束后续各组。与两个 ALS 的情形类似，可以寻找首尾 ALS 的共同候选，并从同时看见首尾两组该候选所有位置的外部格子删除它。']),
}

# Original names are retained both in titles and in the translated prose.
NAMES = {
    '近似锁定数组': 'Almost Locked Set', '锁定数组': 'Locked Set',
    '显性单数': 'Naked Single', '隐性单数': 'Hidden Single',
    '显性数对': 'Naked Pair', '隐性数对': 'Hidden Pair',
    '显性三数组': 'Naked Triple', '隐性三数组': 'Hidden Triple',
    '显性四数组': 'Naked Quad', '隐性四数组': 'Hidden Quad',
    '区块摒除': 'Locked Candidates', '带鳍 X 翼': 'Finned X-Wing',
    '带鳍剑鱼': 'Finned Swordfish', '带鳍水母': 'Finned Jellyfish',
    '带鳍五阶鱼': 'Finned Squirmbag', '五阶鱼': 'Squirmbag',
    '剑鱼': 'Swordfish', '水母': 'Jellyfish', 'WXYZ 翼': 'WXYZ-Wing',
    'XYZ 翼': 'XYZ-Wing', 'XY 翼': 'XY-Wing', 'X 翼': 'X-Wing',
    'ALS 链': 'ALS-Chain',
}
CSS = '''body{margin:8px;background:#fff;color:#000;font:13px "Courier New",monospace}p,li{margin:0 0 12px}h1,h2{font-size:13px;margin:0 0 12px}ul{padding-left:40px}img{display:block;max-width:100%;height:auto;margin:0 0 12px}.back{text-align:right;font-size:smaller}'''

def page(title, body):
    return f'''<!doctype html>
<html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover"><title>{escape(title)}</title><link rel="stylesheet" href="help.css"></head><body>{body}</body></html>\n'''

def original_names(text):
    for zh in sorted(NAMES, key=len, reverse=True):
        text = text.replace(zh, ' ' + NAMES[zh] + ' ')
    return text

def links(*names):
    return '、'.join(f'<a href="{name}.html">{escape(PAGES[name][1])}</a>' for name in names)

def build():
    OUT.mkdir(exist_ok=True)
    (OUT / 'help.css').write_text(CSS, encoding='utf-8')
    used_images = set()
    back = '<p class="back">返回<a href="index.html">课程目录</a></p>'
    for name, (_, en, paragraphs) in PAGES.items():
        body = back + f'<h1>{escape(en)}</h1>'
        for p in paragraphs:
            if p.startswith('@'):
                image = p[1:]
                used_images.add(image)
                body += f'<img src="{escape(image)}" alt="{escape(en)}示例" loading="lazy">'
            else:
                body += f'<p>{original_names(p)}</p>'
        (OUT / f'{name}.html').write_text(page(en, body + back), encoding='utf-8')
    # Preserve the original course-catalog structure, adding links to the two
    # existing pages (Locked Set and ALS-Chain) absent from its original index.
    body = f'''<p class="back"><a href="../index.html">返回主页</a></p>
<h1>SUDOKU 101 BEGINNER SUDOKU</h1><ul>
<li>先修要求：智商高于室温。</li>
<li>通过演绎逻辑和模式识别初识数独；需要专注、耐心和敏锐的观察力。内容包括 Naked Single、Hidden Single、Naked Pair、Hidden Pair 和 Locked Candidates。</li>
<li>入门：{links('full_house','naked_single','hidden_single')}</li>
<li>中等：{links('naked_pair','hidden_pair','locked_candidates','locked_set')}</li>
<li>70191 周一至周四 19:00–20:00</li></ul>
<h2>SUDOKU 401 ADVANCED SUDOKU</h2><ul>
<li>先修要求：智商高于沸水。</li>
<li>学习新的数独技巧和术语——这些名字显然来自钓鱼爱好者和《星球大战》粉丝的想象。需要咬铅笔苦思。</li>
<li>挑战：{links('naked_triple','hidden_triple','naked_quad','hidden_quad','x_wing')}</li>
<li>困难：{links('swordfish','jellyfish','xy_wing','finned_x_wing')}</li>
<li>专家：{links('squirmbag','finned_swordfish','finned_jellyfish')}、Sashimi Fish、Fishy Cycle、{links('xyz_wing','wxyz_wing')}</li>
<li>极难：{links('finned_squirmbag','als')}</li>
<li>70439 周一至周四 20:00–21:00</li></ul>
<h2>SUDOKU 601 SUDOKU INDEPENDENT STUDY</h2><ul>
<li>先修要求：智商高于喷气发动机。</li>
<li>涉及超出一般人类能力的高级数独。需要极客。</li>
<li>烧脑：ALS-XY-Wing、ALS-XY-Chain、Death Blossom、Multi Colors、{links('als_chain')}</li>
<li>70701 待定</li></ul>'''
    (OUT / 'index.html').write_text(page('课程目录', body), encoding='utf-8')
    for image in used_images:
        shutil.copyfile(SOURCE / image, OUT / image)
    print(f'Generated {len(PAGES)} translated pages and copied {len(used_images)} illustrations.')

if __name__ == '__main__':
    build()
