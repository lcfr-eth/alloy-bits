use evmole;
use evmole::control_flow_graph::{Block, BlockType};
use hex;
use std::collections::BTreeMap;
use std::collections::HashSet;

// Structure to represent traced function execution
struct FunctionTrace {
    selector: Vec<u8>,
    selector_hex: String,
    entry_point: usize,
    blocks: Vec<(usize, usize)>,
    disassembled: Vec<(usize, String)>,
}

// Pattern search result with context
struct PatternMatch {
    function_selector: String,
    location: usize,
    context: Vec<(usize, String)>,
}

// Function to search for a specific bytecode pattern in target functions
fn search_pattern_in_functions(
    target_selectors: &[&str],
    function_traces: &[FunctionTrace],
    first_opcode: &str,
    second_opcode: &str,
    context_size: usize,
    pattern_name: &str
) -> Vec<PatternMatch> {
    let mut matches = Vec::new();
    
    // For each function in our traces
    for trace in function_traces {
        // Check if this function is one of our targets
        if target_selectors.contains(&trace.selector_hex.as_str()) {
            let disassembled = &trace.disassembled;
            
            // Look for the pattern in the disassembled code
            for i in 0..disassembled.len() {
                // Check if current instruction contains the first part of pattern
                if disassembled[i].1.contains(first_opcode) {
                    // Look ahead for the second part (within a reasonable distance)
                    for j in i+1..std::cmp::min(disassembled.len(), i+10) {
                        if disassembled[j].1.contains(second_opcode) {
                            // Found a match! Get the context (instructions around the match)
                            let start_idx = if i > context_size { i - context_size } else { 0 };
                            let end_idx = std::cmp::min(j + context_size, disassembled.len() - 1);
                            
                            let context = disassembled[start_idx..=end_idx].to_vec();
                            
                            matches.push(PatternMatch {
                                function_selector: trace.selector_hex.clone(),
                                location: disassembled[i].0,
                                context,
                            });
                            
                            // We found the pattern, no need to keep searching after the second opcode
                            break;
                        }
                    }
                }
            }
        }
    }
    
    matches
}

// Function to trace execution path through CFG starting from entry_point
fn trace_function_execution(
    entry_point: usize,
    blocks: &BTreeMap<usize, Block>,
    disassembled: &Vec<(usize, String)>
) -> (Vec<(usize, usize)>, Vec<(usize, String)>) {
    let mut visited_blocks = HashSet::new();
    let mut blocks_in_order = Vec::new();
    let mut disassembled_in_order = Vec::new();
    let mut block_queue = vec![entry_point];
    
    // Maximum depth to prevent infinite loops in case of cyclic paths
    // This is a safety measure
    let max_depth = 100;
    let mut depth = 0;
    
    while let Some(block_start) = block_queue.pop() {
        // Skip if we've already visited this block or reached max depth
        if visited_blocks.contains(&block_start) || depth > max_depth {
            continue;
        }
        
        // Mark this block as visited
        visited_blocks.insert(block_start);
        depth += 1;
        
        // Find the block in the CFG
        if let Some(block) = blocks.get(&block_start) {
            // Add this block to our execution path
            blocks_in_order.push((block.start, block.end));
            
            // Add the block's instructions to our disassembled path
            let instructions: Vec<(usize, String)> = disassembled
                .iter()
                .filter(|(offset, _)| *offset >= block.start && *offset <= block.end)
                .cloned()
                .collect();
            
            disassembled_in_order.extend(instructions);
            
            // Follow the block's control flow
            match &block.btype {
                BlockType::Jump { to } => {
                    // Direct jump
                    block_queue.push(*to);
                },
                BlockType::Jumpi { true_to, false_to } => {
                    // Conditional jump - follow both paths
                    block_queue.push(*true_to);
                    block_queue.push(*false_to);
                },
                BlockType::DynamicJump { to } => {
                    // Dynamic jump - try to follow all possible paths
                    for jump in to {
                        if let Some(dest) = jump.to {
                            block_queue.push(dest);
                        }
                    }
                },
                BlockType::DynamicJumpi { true_to, false_to } => {
                    // Dynamic conditional jump - follow both paths
                    for jump in true_to {
                        if let Some(dest) = jump.to {
                            block_queue.push(dest);
                        }
                    }
                    block_queue.push(*false_to);
                },
                BlockType::Terminate { .. } => {
                    // End of execution path
                    continue;
                }
            }
        }
    }
    
    // Sort blocks_in_order by start offset to get a more readable output
    blocks_in_order.sort_by_key(|(start, _)| *start);
    
    // Sort disassembled_in_order by offset
    disassembled_in_order.sort_by_key(|(offset, _)| *offset);
    
    // Remove duplicates from disassembled_in_order
    let mut unique_disassembled = Vec::new();
    let mut seen_offsets = HashSet::new();
    
    for (offset, instruction) in disassembled_in_order {
        if seen_offsets.insert(offset) {
            unique_disassembled.push((offset, instruction));
        }
    }
    
    (blocks_in_order, unique_disassembled)
}

pub fn main() {
    // Contract bytecode
    let code = hex::decode(
    //    "608060405234801561000f575f80fd5b506004361061004a575f3560e01c8063771602f71461004e578063893d20e81461006a578063d0f61a1714610088578063fa461e3314610092575b5f80fd5b61006860048036038101906100639190610256565b6100ae565b005b6100726100c4565b60405161007f91906102d3565b60405180910390f35b6100906100eb565b005b6100ac60048036038101906100a79190610380565b610181565b005b80826100ba919061041e565b6001819055505050565b5f805f9054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905090565b5f8054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614610178576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161016f906104ab565b60405180910390fd5b60018081905550565b5f8054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff161461020e576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610205906104ab565b60405180910390fd5b6001808190555050505050565b5f80fd5b5f80fd5b5f819050919050565b61023581610223565b811461023f575f80fd5b50565b5f813590506102508161022c565b92915050565b5f806040838503121561026c5761026b61021b565b5b5f61027985828601610242565b925050602061028a85828601610242565b9150509250929050565b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f6102bd82610294565b9050919050565b6102cd816102b3565b82525050565b5f6020820190506102e65f8301846102c4565b92915050565b5f819050919050565b6102fe816102ec565b8114610308575f80fd5b50565b5f81359050610319816102f5565b92915050565b5f80fd5b5f80fd5b5f80fd5b5f8083601f8401126103405761033f61031f565b5b8235905067ffffffffffffffff81111561035d5761035c610323565b5b60208301915083600182028301111561037957610378610327565b5b9250929050565b5f805f80606085870312156103985761039761021b565b5b5f6103a58782880161030b565b94505060206103b68782880161030b565b935050604085013567ffffffffffffffff8111156103d7576103d661021f565b5b6103e38782880161032b565b925092505092959194509250565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f61042882610223565b915061043383610223565b925082820190508082111561044b5761044a6103f1565b5b92915050565b5f82825260208201905092915050565b7f57726f6e672061646472657373000000000000000000000000000000000000005f82015250565b5f610495600d83610451565b91506104a082610461565b602082019050919050565b5f6020820190508181035f8301526104c281610489565b905091905056fea26469706673582212200a4225b8924828180d8f6cbf62ab784e2d5b4630fed9c49f5baedc16bab48fc364736f6c634300081a0033"
	  "60806040526004361015610011575f80fd5b5f803560e01c806313af40351461118257806323e30c8b14610555578063887d3797146103cb5780639425dd93146102ce578063c5ecf5ea146101905763fa461e331461005c575f80fd5b3461018d57606036600319011261018d57600435816024356044356001600160401b03811161017c576100939036906004016111e4565b83859295138015610184575b1561018057846040918101031261017c576100b9846113f1565b60209490926001600160a01b0391860135918216918290036101785785931561014d575060405163a9059cbb60e01b815233600482015260248101929092529092839190829081604481015b03925af180156101425761011857505080f35b8161013792903d1061013b575b61012f8183611338565b8101906113d9565b5080f35b503d610125565b6040513d85823e3d90fd5b60405163a9059cbb60e01b815233600482015260248101919091529384925082908160448101610105565b8480fd5b8280fd5b8380fd5b5083831361009f565b80fd5b503461018d57604036600319011261018d576001600160401b0360043581811161017c576101c2903690600401611211565b60249291929081358381116102ca576101df903690600401611211565b916101f460018060a01b038854163314611241565b865b818110610201578780f35b61021461020f82848a611288565b6112ac565b848210156102b757888260051b850135601e19863603018112156102b35785019182359089821161017c57602080940191803603831361018057839261025b913691611374565b90828583519301915af161026d6113aa565b5015610282575061027d9061127a565b6101f6565b606490600b876040519262461bcd60e51b845260048401528201526a10d85b1b0819985a5b195960aa1b6044820152fd5b5080fd5b634e487b7160e01b895260326004528589fd5b8580fd5b503461018d57602036600319011261018d576004356001600160401b0381116102b357602061030360a49236906004016111e4565b92839161031a60018060a01b038754163314611241565b604051632e7ff4ef60e11b8152306004820152736b175474e89094c44da98b954eedeac495271d0f60248201526a52b7d2dcc80cd2e4000000604482015260806064820152608481018490529485938492848401378181018301879052601f01601f19168101030181857360744434d6339a6b27d73d9eda62b6f66a0a04fa5af180156103c0576103a9575080f35b6101379060203d811161013b5761012f8183611338565b6040513d84823e3d90fd5b503461018d57606036600319011261018d576001600160401b0360043581811161017c576103fd903690600401611211565b91602480358281116102ca57610417903690600401611211565b939094604493843590811161055157610434903690600401611211565b91909561044b60018060a01b038a54163314611241565b885b898382106104585780f35b808087898c946104c58f61048e898d61048861020f8f8f9061048261020f8780946104d39b611288565b9b611288565b9a611288565b6040805163095ea7b360e01b60208083019182526001600160a01b03909c169782019788529235878c015291959193849290910190565b03601f198101835282611338565b51925af16104df6113aa565b81610521575b50156104fa57506104f59061127a565b61044d565b60649061534160f01b8960028a6040519462461bcd60e51b86526004860152840152820152fd5b80518015925083908315610539575b5050505f6104e5565b61054993508201810191016113d9565b5f8281610530565b8780fd5b503461018d5760a036600319011261018d5761056f6111ce565b6024356001600160a01b038116036102b3576001600160401b0390608435828111610180576105a29036906004016111e4565b9290917360744434d6339a6b27d73d9eda62b6f66a0a04fa330361113d57306001600160a01b03909116036110e85783928201916060818403126109c2576105e9816113f1565b9260208201358381116102ca57816106029184016113fe565b9260408301359081116102ca5761061992016113fe565b91156109f15760a0818051810103126108f75760405190610639826112c0565b6020810151825261064c6040820161141c565b90602083019182526106606060820161141c565b604084015260a060808201519160608501928352015191608084019283525f805160206119eb8339815191523b156102ca5760405163617ba03760e01b81528681806106af3060048301611430565b0381835f805160206119eb8339815191525af19081156109e65787916109d2575b50505190516001600160a01b03909116905f805160206119eb8339815191523b156102ca576040519163a415bcad60e01b83526004830152602482015260026044820152846064820152306084820152848160a481835f805160206119eb8339815191525af19081156109c75785916109ae575b505061075360249383516114c5565b60408281015190516370a0823160e01b81523060048201529360209185919082906001600160a01b03165afa92831561096c578493610977575b5073c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2908160018060a01b03604085015116036108fb575b5050604001516001600160a01b0316905f805160206119eb8339815191523b156108f7576040519163617ba03760e01b8352600483015260248201523060448201525f60648201528181608481835f805160206119eb8339815191525af180156103c0576108e3575b5050604051631a4ca37b60e21b8152736b175474e89094c44da98b954eedeac495271d0f60048201525f19602482015230604482015290602082606481845f805160206119eb8339815191525af19081156108d757506108a8575b505b60206040517f439148f0bbc682ca079e46d6e2c2f0c1e3b820f1a291b069d8882abf8cf18dd98152f35b602090813d83116108d0575b6108be8183611338565b810103126108cc575f61087c565b5f80fd5b503d6108b4565b604051903d90823e3d90fd5b6108ec906112ef565b61018d57805f610821565b5050fd5b515f805160206119eb8339815191523b15610178576040519163617ba03760e01b8352600483015260248201523060448201525f60648201528381608481835f805160206119eb8339815191525af190811561096c578491156107b857610961906112ef565b6108f757825f6107b8565b6040513d86823e3d90fd5b935091506020833d6020116109a6575b8161099460209383611338565b810103126108cc57839251915f61078d565b3d9150610987565b6109b7906112ef565b6109c257835f610744565b505050fd5b6040513d87823e3d90fd5b6109db906112ef565b6102ca57855f6106d0565b6040513d89823e3d90fd5b915060a08280518101031261017c5760405190610a0d826112c0565b60208301518252610a206040840161141c565b6020830152610a316060840161141c565b6040830152610a4e60a0608085015194606085019586520161141c565b60808301525f805160206119eb8339815191523b156101805760405163617ba03760e01b8152848180610a843060048301611430565b0381835f805160206119eb8339815191525af180156109c7576110d5575b506040828101519051631a4ca37b60e21b81526001600160a01b0390911660048201525f196024820152306044820152602081606481885f805160206119eb8339815191525af180156109c7576110aa575b5060408201516001600160a01b031673c02aaa39b223fe8d0a0e5c4f27ead9083c756cc11901611023575b610b2a9082516114c5565b60208181015160405163573ade8160e01b81526001600160a01b0390911660048201525f196024820152600260448201523060648201529081608481875f805160206119eb8339815191525af1801561096c57610ff8575b50604051631a4ca37b60e21b8152736b175474e89094c44da98b954eedeac495271d0f60048201526a52b7d2dcc80cd2e40000006024820152306044820152602081606481875f805160206119eb8339815191525af1801561096c57610fcd575b5060208101516001600160a01b03169173c02aaa39b223fe8d0a0e5c4f27ead9083c756cc1198301610c19575b5050505061087e565b516080909101516040516370a0823160e01b81523060048201526001600160a01b0391909116929091602083602481855afa9283156109c7578593610f99575b5080610ed05750604051630240bc6b60e21b815291606083600481875afa9283156109c75785908694610e78575b506001600160701b03938416931673c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2831015610e725792915b8115610e19578315801580610e10575b15610dba576103e590818402918483048103610da65785850202948286041482151715610d92576103e8808702968704141715610d7e578401809411610d6a578315610d5657610d4d955060405194610d1c866112c0565b8552602085015273c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2604085015260608401520460808201526117d4565b5f808080610c10565b634e487b7160e01b86526012600452602486fd5b634e487b7160e01b86526011600452602486fd5b634e487b7160e01b87526011600452602487fd5b634e487b7160e01b5f52601160045260245ffd5b634e487b7160e01b89526011600452602489fd5b60405162461bcd60e51b815260206004820152602860248201527f556e697377617056324c6962726172793a20494e53554646494349454e545f4c604482015267495155494449545960c01b6064820152608490fd5b50831515610cc4565b60405162461bcd60e51b815260206004820152602b60248201527f556e697377617056324c6962726172793a20494e53554646494349454e545f4960448201526a1394155517d05353d5539560aa1b6064820152608490fd5b91610cb4565b9350506060833d606011610ec8575b81610e9460609383611338565b8101031261017857610ea5836117c0565b6040610eb3602086016117c0565b94015163ffffffff8116036102ca575f610c87565b3d9150610e87565b90935060018103610f1d5750610f189260405192610eed84611302565b8352602083015273c02aaa39b223fe8d0a0e5c4f27ead9083c756cc260408301526060820152611900565b610d4d565b600203610f6057610f189260405192610f3584611302565b8352602083015273c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2604083015260608201526115cd565b60405162461bcd60e51b8152602060048201526011602482015270155b9cdd5c1c1bdc9d19590819195e1259607a1b6044820152606490fd5b9092506020813d602011610fc5575b81610fb560209383611338565b810103126108cc5751915f610c59565b3d9150610fa8565b602090813d8311610ff1575b610fe38183611338565b810103126108cc575f610be3565b503d610fd9565b602090813d831161101c575b61100e8183611338565b810103126108cc575f610b82565b503d611004565b604051631a4ca37b60e21b815273c02aaa39b223fe8d0a0e5c4f27ead9083c756cc260048201525f196024820152306044820152602081606481885f805160206119eb8339815191525af180156109c75761107f575b50610b1f565b602090813d83116110a3575b6110958183611338565b810103126108cc575f611079565b503d61108b565b602090813d83116110ce575b6110c08183611338565b810103126108cc575f610af4565b503d6110b6565b6110e1909491946112ef565b925f610aa2565b60405162461bcd60e51b815260206004820152602760248201527f466c617368426f72726f7765723a20556e74727573746564206c6f616e20696e60448201526634ba34b0ba37b960c91b6064820152608490fd5b60405162461bcd60e51b815260206004820152601f60248201527f466c617368426f72726f7765723a20556e74727573746564206c656e646572006044820152606490fd5b503461018d57602036600319011261018d5761119c6111ce565b8154906001600160a01b03906111b53383851614611241565b16906bffffffffffffffffffffffff60a01b1617815580f35b600435906001600160a01b03821682036108cc57565b9181601f840112156108cc578235916001600160401b0383116108cc57602083818601950101116108cc57565b9181601f840112156108cc578235916001600160401b0383116108cc576020808501948460051b0101116108cc57565b1561124857565b60405162461bcd60e51b815260206004820152600a60248201526927b7363c9037bbb732b960b11b6044820152606490fd5b5f198114610d925760010190565b91908110156112985760051b0190565b634e487b7160e01b5f52603260045260245ffd5b356001600160a01b03811681036108cc5790565b60a081019081106001600160401b038211176112db57604052565b634e487b7160e01b5f52604160045260245ffd5b6001600160401b0381116112db57604052565b608081019081106001600160401b038211176112db57604052565b602081019081106001600160401b038211176112db57604052565b90601f801991011681019081106001600160401b038211176112db57604052565b6001600160401b0381116112db57601f01601f191660200190565b92919261138082611359565b9161138e6040519384611338565b8294818452818301116108cc578281602093845f960137010152565b3d156113d4573d906113bb82611359565b916113c96040519384611338565b82523d5f602084013e565b606090565b908160209103126108cc575180151581036108cc5790565b359081151582036108cc57565b9080601f830112156108cc5781602061141993359101611374565b90565b51906001600160a01b03821682036108cc57565b736b175474e89094c44da98b954eedeac495271d0f81526a52b7d2dcc80cd2e400000060208201526001600160a01b0390911660408201525f606082015260800190565b91908260809103126108cc5760405161148c81611302565b606080829461149a8161141c565b84526114a86020820161141c565b60208501526114b96040820161141c565b60408501520151910152565b80611533575060a0818051810103126108cc576115319060a0604051916114eb836112c0565b6114f76020820161141c565b83526115056040820161141c565b60208401526115166060820161141c565b604084015260808101516060840152015160808201526117d4565b565b600181036115625750805181016080828203126108cc576115319160208061155d93019101611474565b611900565b600203610f6057805181016080828203126108cc576115319160208061158a93019101611474565b6115cd565b91908251928382525f5b8481106115b9575050825f602080949584010152601f8019910116010190565b602081830181015184830182015201611599565b80516040805163038fff2d60e41b815260209390926001600160a01b03919085908590600490829086165afa9384156117b6575f94611787575b508185820151166060838584015116920151918451936116268561131d565b5f855285519060c08201978289106001600160401b038a11176112db57600498885282528882015f8152878301948552606083019384526080830195865260a0830196875287519361167785611302565b3085528a8501905f82528986019230845260608701955f875260e08c519d8e6352bbbe29831b815201525160e48d0152516002811015611773578c998c99868094818d9c6116f0966101048f015251166101248d015251166101448b0152516101648a01525160c06101848a01526101a489019061158f565b955116602487015251151560448601525116606484015251151560848301525f60a48301525f1960c483015203815f73ba12222222228d8ba445958a75a0704d566bf2c85af190811561176a5750611746575050565b813d8311611763575b6117598183611338565b810103126108cc57565b503d61174f565b513d5f823e3d90fd5b634e487b7160e01b5f52602160045260245ffd5b90938582813d83116117af575b61179e8183611338565b8101031261018d575051925f611607565b503d611794565b83513d5f823e3d90fd5b51906001600160701b03821682036108cc57565b80516020808301805160608501516040805163a9059cbb60e01b81526001600160a01b03968716600482018190526024820193909352928616979690959194909391929081806044810103815f809c5af180156118f6579183929160809594926118d8575b505116908583015116115f146118cf5701519084915b83519261185b8461131d565b868452823b156118cb5791869391846118a497989487519889958694859363022c0d9f60e01b85526004850152602484015230604484015260806064840152608483019061158f565b03925af19182156118c15750506118b85750565b611531906112ef565b51903d90823e3d90fd5b8680fd5b0151908461184f565b6118ef9060203d811161013b5761012f8183611338565b505f611839565b86513d8a823e3d90fd5b8051602082015160408084015190936001600160a01b0392831693918316841091831682156119cf576401000276ad935b8651958460208801528787015286865260608601948686106001600160401b038711176112db57879460608795868852015190630251596160e31b86523060648a0152608489015260a48801521660c486015260a060e4860152815f605f198761199f61010482018261158f565b0301925af180156117b6576119b357505050565b82903d84116119c7575b8161175991611338565b3d91506119bd565b73fffd8963efd1fc6a506488495d951d5263988d259361193156fe00000000000000000000000087870bca3f3fd6335c3f4ce8392d69350b4fa4e2a2646970667358221220cf4f7321130d39a21cd1461f527aeb5ecd6a277c6c2797246e7af3409306af9c64736f6c63430008140033"
    ).unwrap();

    let evmole_info = evmole::contract_info(
        evmole::ContractInfoArgs::new(&code)
            .with_selectors()
            .with_arguments()
            .with_storage()
            .with_disassemble()
            .with_basic_blocks()
            .with_control_flow_graph()
            .with_state_mutability(),
    );

    // Print contract overview
    println!("======= CONTRACT ANALYSIS =======\n");

    // Print Functions and trace their execution paths
    let mut function_traces = Vec::new();
    
    if let Some(funcs) = &evmole_info.functions {
        println!("======= FUNCTIONS =======");
        println!("{:<12} {:<12} {:<30} {:<15}", "Selector", "Offset", "Arguments", "Mutability");
        println!("{}", "-".repeat(70));
        
        for function in funcs {
            let selector_hex = format!("0x{}", hex::encode(&function.selector));
            let arguments = if let Some(args) = &function.arguments {
                args.iter()
                    .map(|arg| format!("{:?}", arg))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                "None".to_string()
            };

            let state_mutability = function.state_mutability
                .as_ref()
                .map(|s| format!("{:?}", s))
                .unwrap_or("Unknown".to_string());

            println!(
                "{:<12} {:<12} {:<30} {:<15}",
                selector_hex, function.bytecode_offset, arguments, state_mutability
            );
            
            // Create function trace for later analysis
            if let Some(cfg) = &evmole_info.control_flow_graph {
                if let Some(disassembled) = &evmole_info.disassembled {
                    let traced_blocks = trace_function_execution(
                        function.bytecode_offset, 
                        &cfg.blocks, 
                        disassembled
                    );
                    
                    function_traces.push(FunctionTrace {
                        selector: function.selector.to_vec(),
                        selector_hex: selector_hex.clone(),
                        entry_point: function.bytecode_offset,
                        blocks: traced_blocks.0,
                        disassembled: traced_blocks.1,
                    });
                }
            }
        }
        println!();
    } else {
        println!("No functions found.\n");
    }
    
    // Perform pattern searches
    let target_selectors = vec!["0xfa461e33", "0xd0f61a17"]; // Add selectors to target
    
    // Search for authentication patterns (CALLER + EQ)
    let auth_matches = search_pattern_in_functions(
        &target_selectors,
        &function_traces,
        "CALLER",
        "EQ",
        5, // Context window size
        "Authorization Check (CALLER+EQ)"
    );
    if !auth_matches.is_empty() {
        println!("======= PATTERN SEARCH: AUTHORIZATION CHECKS =======");
        for (i, pattern_match) in auth_matches.iter().enumerate() {
            println!("\nMatch #{} in function {} at offset 0x{:x}:", 
                    i+1, pattern_match.function_selector, pattern_match.location);
            println!("Context:");
            for (offset, instruction) in &pattern_match.context {
                println!("  0x{:<8x} {:<20}", offset, instruction);
            }
        }
        println!();
    }

    if auth_matches.is_empty() {
        println!("======= NO AUTH CHECKS FOUND =======");
    }


/*

    // Print Function Blocks Analysis
    if !function_traces.is_empty() {
        println!("======= FUNCTION BLOCKS ANALYSIS =======");
        
        for trace in &function_traces {
            println!("\nFunction: {} (offset 0x{:x})", trace.selector_hex, trace.entry_point);
            println!("{}", "-".repeat(50));
            
            println!("Execution Blocks:");
            for (i, (start, end)) in trace.blocks.iter().enumerate() {
                println!("  Block {}: 0x{:x} - 0x{:x}", i+1, start, end);
            }
            
            println!("\nDisassembled Code Path:");
            for (offset, instruction) in &trace.disassembled {
                // Check if this offset is the start of a block to add visual separation
                if trace.blocks.iter().any(|(start, _)| start == offset) {
                    println!("\n  --- New Block ---");
                }
                println!("  0x{:<8x} {:<20}", offset, instruction);
            }
            println!();
        }
    }
    
    // Print Storage Information
    if let Some(storage) = &evmole_info.storage {
        println!("======= STORAGE RECORDS =======");
        println!("{:<66} {:<8} {:<10} {:<20} {:<20}", "Slot", "Offset", "Type", "Reads", "Writes");
        println!("{}", "-".repeat(120));
        
        for record in storage {
            let slot_hex = format!("0x{}", hex::encode(&record.slot));
            let reads = record
                .reads
                .iter()
                .map(|selector| format!("0x{}", hex::encode(selector)))
                .collect::<Vec<_>>()
                .join(", ");

            let writes = record
                .writes
                .iter()
                .map(|selector| format!("0x{}", hex::encode(selector)))
                .collect::<Vec<_>>()
                .join(", ");

            println!(
                "{:<66} {:<8} {:<10} {:<20} {:<20}",
                slot_hex, record.offset, record.r#type, reads, writes
            );
        }
        println!();
    } else {
        println!("No storage records found.\n");
    }

    // Print Basic Blocks
    if let Some(basic_blocks) = &evmole_info.basic_blocks {
        println!("======= BASIC BLOCKS =======");
        println!("{:<8} {:<8} {:<10}", "Start", "End", "Size");
        println!("{}", "-".repeat(30));

        // Convert to sorted map for better display
        let mut blocks_map: BTreeMap<usize, usize> = BTreeMap::new();
        for (start, end) in basic_blocks {
            blocks_map.insert(*start, *end);
        }

        for (start, end) in blocks_map.iter() {
            let size = end - start + 1;
            println!("{:<8} {:<8} {:<10}", start, end, size);
        }
        println!();
    } else {
        println!("No basic blocks found.\n");
    }

    // Print Control Flow Graph
    if let Some(cfg) = &evmole_info.control_flow_graph {
        println!("======= CONTROL FLOW GRAPH =======");
        println!("{:<8} {:<8} {:<8} {:<50}", "Block", "Start", "End", "Flow Type");
        println!("{}", "-".repeat(80));

        // Sort blocks by their start position for cleaner output
        let mut sorted_blocks: Vec<(&usize, &Block)> = cfg.blocks.iter().collect();
        sorted_blocks.sort_by_key(|(id, _)| *id);

        for (block_id, block) in sorted_blocks {
            let flow_type = match &block.btype {
                BlockType::Jump { to } => format!("Jump to {}", to),
                BlockType::Jumpi { true_to, false_to } => 
                    format!("Conditional: true->{}, false->{}", true_to, false_to),
                BlockType::DynamicJump { to } => {
                    let destinations: Vec<String> = to.iter()
                        .filter_map(|jump| jump.to.map(|t| t.to_string()))
                        .collect();
                    format!("Dynamic: [{}]", destinations.join(", "))
                },
                BlockType::DynamicJumpi { true_to, false_to } => {
                    let true_destinations: Vec<String> = true_to.iter()
                        .filter_map(|jump| jump.to.map(|t| t.to_string()))
                        .collect();
                    format!("Dynamic Conditional: true->[{}], false->{}", 
                            true_destinations.join(", "), false_to)
                },
                BlockType::Terminate { success } => 
                    format!("Terminate ({})", if *success { "success" } else { "failure" }),
            };

            println!(
                "{:<8} {:<8} {:<8} {:<50}",
                block_id, block.start, block.end, flow_type
            );
        }
        println!();
    } else {
        println!("No control flow graph found.\n");
    }

    // Print Disassembled Bytecode
    if let Some(disassembled) = &evmole_info.disassembled {
        println!("======= DISASSEMBLED BYTECODE =======");
        println!("{:<10} {:<20}", "Offset", "Instruction");
        println!("{}", "-".repeat(30));

        // Group disassembled code by basic blocks for better readability
        let mut current_block = 0;
        if let Some(basic_blocks) = &evmole_info.basic_blocks {
            // Convert to sorted map
            let mut blocks_map: BTreeMap<usize, usize> = BTreeMap::new();
            for (start, end) in basic_blocks {
                blocks_map.insert(*start, *end);
            }
            
            let mut block_starts: Vec<usize> = blocks_map.keys().cloned().collect();
            block_starts.sort();

            for (offset, instruction) in disassembled {
                // Check if we're entering a new block
                if block_starts.contains(offset) {
                    current_block += 1;
                    println!("\n--- Block {} ---", current_block);
                }
                
                println!("0x{:<8x} {:<20}", offset, instruction);
            }
        } else {
            // If no basic blocks, just print instructions
            for (offset, instruction) in disassembled {
                println!("0x{:<8x} {:<20}", offset, instruction);
            }
        }
        println!();
    } else {
        println!("No disassembled bytecode found.\n");
    }
*/
}
