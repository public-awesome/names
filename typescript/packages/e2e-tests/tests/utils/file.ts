import fs from 'fs'
import readline from 'readline'

export const readChecksumFile = async (checksumFilePath: string, target: string): Promise<string> => {
  return new Promise((resolve, reject) => {
    let retval: string
    const readInterface = readline.createInterface(fs.createReadStream(checksumFilePath))

    readInterface.on('line', function (line) {
      const [checksum, fileName] = line.split(/\s+/) // Split by whitespace
      if (fileName.trim() === target) {
        retval = checksum
        resolve(retval)
        readInterface.close() // Close the stream if the checksum is found
      }
    })

    readInterface.on('close', function () {
      if (!retval) {
        reject(`Checksum for ${target} not found.`)
      }
    })
  })
}
